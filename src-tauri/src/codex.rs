use serde::Serialize;
use serde_json::{json, Value};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdout, Command},
    time::{self, Duration},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexStreamEvent {
    pub request_id: String,
    pub kind: &'static str,
    pub text: String,
}

#[derive(Debug)]
pub struct CodexError {
    pub code: &'static str,
    pub message: String,
}

impl CodexError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub struct CodexAdapter {
    executable: PathBuf,
    state_dir: PathBuf,
    workspace_dir: PathBuf,
}

impl CodexAdapter {
    pub fn bundled(data_dir: &Path) -> io::Result<Self> {
        let state_dir = data_dir.join("codex").join("state");
        let workspace_dir = data_dir.join("codex").join("empty-workspace");
        fs::create_dir_all(&state_dir)?;
        fs::create_dir_all(&workspace_dir)?;
        Ok(Self {
            executable: codex_executable(),
            state_dir,
            workspace_dir,
        })
    }

    pub async fn review(
        &self,
        request_id: &str,
        package: &str,
        cancelled: Arc<AtomicBool>,
        emit: impl Fn(CodexStreamEvent),
    ) -> Result<String, CodexError> {
        if package.trim().is_empty() {
            return Err(CodexError::new(
                "codex_package_empty",
                "Пакет проверки пуст",
            ));
        }
        let mut child = Command::new(&self.executable)
            .arg("app-server")
            .arg("--listen")
            .arg("stdio://")
            .env("CODEX_HOME", &self.state_dir)
            .current_dir(&self.workspace_dir)
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| {
                CodexError::new(
                    "codex_sidecar_missing",
                    format!("Bundled Codex не найден или не запускается: {error}"),
                )
            })?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            CodexError::new("codex_protocol_failed", "Codex не открыл канал команд")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            CodexError::new("codex_protocol_failed", "Codex не открыл поток ответа")
        })?;
        let mut lines = BufReader::new(stdout).lines();

        send(&mut stdin, json!({
            "method": "initialize",
            "id": 1,
            "params": { "clientInfo": { "name": "bookshelf", "title": "Bookshelf", "version": env!("CARGO_PKG_VERSION") } }
        })).await?;
        response(&mut lines, 1).await?;
        send(&mut stdin, json!({ "method": "initialized", "params": {} })).await?;
        send(
            &mut stdin,
            json!({
                "method": "thread/start",
                "id": 2,
                "params": {
                    "cwd": self.workspace_dir,
                    "approvalPolicy": "never",
                    "sandbox": "readOnly",
                    "serviceName": "bookshelf"
                }
            }),
        )
        .await?;
        let thread = response(&mut lines, 2).await?;
        let thread_id = thread
            .pointer("/result/thread/id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул несовместимый ответ thread/start",
                )
            })?;
        let prompt = format!(
            "Ты внешний критик уже сформулированной читателем идеи. Не используй инструменты, файлы или сеть. Не переписывай идею за читателя и не выставляй итоговую оценку. Укажи возможные пробелы, ограничения и вопросы только по подтверждённому пакету ниже.\n\n{package}"
        );
        send(&mut stdin, json!({
            "method": "turn/start",
            "id": 3,
            "params": {
                "threadId": thread_id,
                "input": [{ "type": "text", "text": prompt }],
                "approvalPolicy": "never",
                "sandboxPolicy": {
                    "type": "readOnly",
                    "access": { "type": "restricted", "includePlatformDefaults": true, "readableRoots": [] }
                }
            }
        })).await?;
        let turn = response(&mut lines, 3).await?;
        let turn_id = turn
            .pointer("/result/turn/id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул несовместимый ответ turn/start",
                )
            })?
            .to_owned();
        let mut answer = String::new();
        let mut interrupt_sent = false;
        loop {
            if cancelled.load(Ordering::Relaxed) && !interrupt_sent {
                send(
                    &mut stdin,
                    json!({
                        "method": "turn/interrupt",
                        "id": 4,
                        "params": { "threadId": thread_id, "turnId": turn_id }
                    }),
                )
                .await?;
                interrupt_sent = true;
            }
            let line = match time::timeout(Duration::from_millis(150), lines.next_line()).await {
                Err(_) => continue,
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => {
                    return Err(CodexError::new(
                        "codex_crashed",
                        "Codex завершился до окончания проверки",
                    ))
                }
                Ok(Err(error)) => {
                    return Err(CodexError::new(
                        "codex_protocol_failed",
                        format!("Не удалось прочитать ответ Codex: {error}"),
                    ))
                }
            };
            let value: Value = serde_json::from_str(&line).map_err(|_| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул неизвестный формат события",
                )
            })?;
            match stream_message(&value) {
                StreamMessage::Delta(delta) => {
                    answer.push_str(delta);
                    emit(CodexStreamEvent {
                        request_id: request_id.into(),
                        kind: "delta",
                        text: delta.into(),
                    });
                }
                StreamMessage::Completed => {
                    if answer.trim().is_empty() {
                        return Err(CodexError::new(
                            "codex_response_empty",
                            "Codex завершил проверку без текста",
                        ));
                    }
                    return Ok(answer);
                }
                StreamMessage::Interrupted => {
                    return Err(CodexError::new("codex_cancelled", "Проверка отменена"))
                }
                StreamMessage::Failed(message) => {
                    return Err(CodexError::new("codex_review_failed", message))
                }
                StreamMessage::Ignore => {}
            }
        }
    }

    pub async fn login(&self, emit: impl Fn(CodexStreamEvent)) -> Result<(), CodexError> {
        let mut child = Command::new(&self.executable)
            .arg("app-server")
            .arg("--listen")
            .arg("stdio://")
            .env("CODEX_HOME", &self.state_dir)
            .current_dir(&self.workspace_dir)
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|error| {
                CodexError::new(
                    "codex_sidecar_missing",
                    format!("Bundled Codex не найден или не запускается: {error}"),
                )
            })?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            CodexError::new("codex_protocol_failed", "Codex не открыл канал команд")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            CodexError::new("codex_protocol_failed", "Codex не открыл поток ответа")
        })?;
        let mut lines = BufReader::new(stdout).lines();
        send(&mut stdin, json!({"method":"initialize","id":1,"params":{"clientInfo":{"name":"bookshelf","title":"Bookshelf","version":env!("CARGO_PKG_VERSION")}}})).await?;
        response(&mut lines, 1).await?;
        send(&mut stdin, json!({"method":"initialized","params":{}})).await?;
        send(
            &mut stdin,
            json!({"method":"account/login/start","id":2,"params":{"type":"chatgptDeviceCode"}}),
        )
        .await?;
        let login = response(&mut lines, 2).await?;
        let verification_url = login
            .pointer("/result/verificationUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodexError::new("codex_protocol_incompatible", "Codex не вернул адрес входа")
            })?;
        let user_code = login
            .pointer("/result/userCode")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodexError::new("codex_protocol_incompatible", "Codex не вернул код входа")
            })?;
        emit(CodexStreamEvent {
            request_id: "login".into(),
            kind: "deviceCode",
            text: format!("{verification_url}\n{user_code}"),
        });
        loop {
            let line = lines
                .next_line()
                .await
                .map_err(|error| CodexError::new("codex_protocol_failed", error.to_string()))?
                .ok_or_else(|| {
                    CodexError::new("codex_crashed", "Codex завершился во время входа")
                })?;
            let value: Value = serde_json::from_str(&line).map_err(|_| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул неизвестный формат события входа",
                )
            })?;
            if value.get("method").and_then(Value::as_str) != Some("account/login/completed") {
                continue;
            }
            if value.pointer("/params/success").and_then(Value::as_bool) == Some(true) {
                return Ok(());
            }
            return Err(CodexError::new(
                "codex_login_failed",
                value
                    .pointer("/params/error")
                    .and_then(Value::as_str)
                    .unwrap_or("Вход в Codex не завершён"),
            ));
        }
    }
}

async fn send(stdin: &mut tokio::process::ChildStdin, value: Value) -> Result<(), CodexError> {
    let mut encoded = serde_json::to_vec(&value)
        .map_err(|error| CodexError::new("codex_protocol_failed", error.to_string()))?;
    encoded.push(b'\n');
    stdin.write_all(&encoded).await.map_err(|error| {
        CodexError::new(
            "codex_crashed",
            format!("Codex закрыл канал команд: {error}"),
        )
    })
}

async fn response(lines: &mut Lines<BufReader<ChildStdout>>, id: u64) -> Result<Value, CodexError> {
    loop {
        let line = lines
            .next_line()
            .await
            .map_err(|error| CodexError::new("codex_protocol_failed", error.to_string()))?
            .ok_or_else(|| CodexError::new("codex_crashed", "Codex завершился во время запуска"))?;
        let value: Value = serde_json::from_str(&line).map_err(|_| {
            CodexError::new(
                "codex_protocol_incompatible",
                "Codex вернул неизвестный формат ответа",
            )
        })?;
        if value.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }
        if let Some(error) = value.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Неизвестная ошибка Codex");
            let code = if message.to_lowercase().contains("auth")
                || message.to_lowercase().contains("login")
            {
                "codex_login_required"
            } else {
                "codex_protocol_incompatible"
            };
            return Err(CodexError::new(code, message));
        }
        return Ok(value);
    }
}

enum StreamMessage<'a> {
    Delta(&'a str),
    Completed,
    Interrupted,
    Failed(String),
    Ignore,
}

fn stream_message(value: &Value) -> StreamMessage<'_> {
    match value.get("method").and_then(Value::as_str) {
        Some("item/agentMessage/delta") => value
            .pointer("/params/delta")
            .and_then(Value::as_str)
            .map(StreamMessage::Delta)
            .unwrap_or(StreamMessage::Ignore),
        Some("turn/completed") => {
            match value.pointer("/params/turn/status").and_then(Value::as_str) {
                Some("completed") => StreamMessage::Completed,
                Some("interrupted") => StreamMessage::Interrupted,
                Some("failed") => StreamMessage::Failed(
                    value
                        .pointer("/params/turn/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("Проверка Codex завершилась с ошибкой")
                        .into(),
                ),
                _ => StreamMessage::Ignore,
            }
        }
        Some("error") => StreamMessage::Failed(
            value
                .pointer("/params/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Ошибка Codex")
                .into(),
        ),
        _ => StreamMessage::Ignore,
    }
}

fn codex_executable() -> PathBuf {
    if let Some(path) = env::var_os("BOOKSHELF_CODEX_SIDECAR") {
        return path.into();
    }
    if let Ok(current) = env::current_exe() {
        let name = if cfg!(windows) { "codex.exe" } else { "codex" };
        if let Some(parent) = current.parent() {
            let bundled = parent.join(name);
            if bundled.exists() {
                return bundled;
            }
        }
    }
    PathBuf::from(if cfg!(windows) { "codex.exe" } else { "codex" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_maps_stream_deltas_and_terminal_states() {
        let delta = json!({"method":"item/agentMessage/delta","params":{"delta":"Пробел"}});
        assert!(matches!(
            stream_message(&delta),
            StreamMessage::Delta("Пробел")
        ));
        let complete = json!({"method":"turn/completed","params":{"turn":{"status":"completed"}}});
        assert!(matches!(
            stream_message(&complete),
            StreamMessage::Completed
        ));
        let interrupted =
            json!({"method":"turn/completed","params":{"turn":{"status":"interrupted"}}});
        assert!(matches!(
            stream_message(&interrupted),
            StreamMessage::Interrupted
        ));
    }
}
