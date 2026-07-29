use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CodexStreamEvent {
    pub request_id: String,
    pub kind: &'static str,
    pub text: String,
}

#[derive(Serialize)]
struct RpcRequest<P> {
    method: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    params: P,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    id: u64,
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Clone, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientInfo {
    name: &'static str,
    title: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    client_info: ClientInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThreadStartParams<'a> {
    cwd: &'a Path,
    approval_policy: &'static str,
    sandbox: &'static str,
    service_name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TurnStartParams<'a> {
    thread_id: &'a str,
    input: [TextInput<'a>; 1],
    approval_policy: &'static str,
    sandbox_policy: SandboxPolicy,
}

#[derive(Serialize)]
struct TextInput<'a> {
    r#type: &'static str,
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SandboxPolicy {
    r#type: &'static str,
    access: SandboxAccess,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SandboxAccess {
    r#type: &'static str,
    include_platform_defaults: bool,
    readable_roots: [String; 0],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InterruptParams<'a> {
    thread_id: &'a str,
    turn_id: &'a str,
}

#[derive(Serialize)]
struct LoginStartParams {
    r#type: &'static str,
}

#[derive(Deserialize)]
struct ThreadStartResult {
    thread: ProtocolId,
}

#[derive(Deserialize)]
struct TurnStartResult {
    turn: ProtocolId,
}

#[derive(Deserialize)]
struct ProtocolId {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginStartResult {
    verification_url: String,
    user_code: String,
}

#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
enum ServerEvent {
    #[serde(rename = "item/agentMessage/delta")]
    AgentMessageDelta { delta: String },
    #[serde(rename = "turn/completed")]
    TurnCompleted { turn: CompletedTurn },
    #[serde(rename = "account/login/completed")]
    LoginCompleted {
        success: bool,
        error: Option<String>,
    },
    #[serde(rename = "error")]
    Error { error: RpcError },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct CompletedTurn {
    status: String,
    error: Option<RpcError>,
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
            executable: codex_executable()?,
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
        let (_child, mut stdin, mut lines) = self.connect().await?;
        send(
            &mut stdin,
            &RpcRequest {
                method: "thread/start",
                id: Some(2),
                params: ThreadStartParams {
                    cwd: &self.workspace_dir,
                    approval_policy: "never",
                    sandbox: "readOnly",
                    service_name: "bookshelf",
                },
            },
        )
        .await?;
        let thread: ThreadStartResult = response(&mut lines, 2).await?;
        let thread_id = thread.thread.id;
        let prompt = format!(
            "Ты внешний критик уже сформулированной читателем идеи. Не используй инструменты, файлы или сеть. Не переписывай идею за читателя и не выставляй итоговую оценку. Укажи возможные пробелы, ограничения и вопросы только по подтверждённому пакету ниже.\n\n{package}"
        );
        send(
            &mut stdin,
            &RpcRequest {
                method: "turn/start",
                id: Some(3),
                params: TurnStartParams {
                    thread_id: &thread_id,
                    input: [TextInput {
                        r#type: "text",
                        text: &prompt,
                    }],
                    approval_policy: "never",
                    sandbox_policy: SandboxPolicy {
                        r#type: "readOnly",
                        access: SandboxAccess {
                            r#type: "restricted",
                            include_platform_defaults: true,
                            readable_roots: [],
                        },
                    },
                },
            },
        )
        .await?;
        let turn: TurnStartResult = response(&mut lines, 3).await?;
        let turn_id = turn.turn.id;
        let mut answer = String::new();
        let mut interrupt_sent = false;
        loop {
            if cancelled.load(Ordering::Relaxed) && !interrupt_sent {
                send(
                    &mut stdin,
                    &RpcRequest {
                        method: "turn/interrupt",
                        id: Some(4),
                        params: InterruptParams {
                            thread_id: &thread_id,
                            turn_id: &turn_id,
                        },
                    },
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
            let event: ServerEvent = serde_json::from_str(&line).map_err(|_| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул неизвестный формат события",
                )
            })?;
            match stream_message(event) {
                StreamMessage::Delta(delta) => {
                    answer.push_str(&delta);
                    emit(CodexStreamEvent {
                        request_id: request_id.into(),
                        kind: "delta",
                        text: delta,
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
        let (_child, mut stdin, mut lines) = self.connect().await?;
        send(
            &mut stdin,
            &RpcRequest {
                method: "account/login/start",
                id: Some(2),
                params: LoginStartParams {
                    r#type: "chatgptDeviceCode",
                },
            },
        )
        .await?;
        let login: LoginStartResult = response(&mut lines, 2).await?;
        emit(CodexStreamEvent {
            request_id: "login".into(),
            kind: "deviceCode",
            text: format!("{}\n{}", login.verification_url, login.user_code),
        });
        loop {
            let line = lines
                .next_line()
                .await
                .map_err(|error| CodexError::new("codex_protocol_failed", error.to_string()))?
                .ok_or_else(|| {
                    CodexError::new("codex_crashed", "Codex завершился во время входа")
                })?;
            let event: ServerEvent = serde_json::from_str(&line).map_err(|_| {
                CodexError::new(
                    "codex_protocol_incompatible",
                    "Codex вернул неизвестный формат события входа",
                )
            })?;
            if let ServerEvent::LoginCompleted { success, error } = event {
                if success {
                    return Ok(());
                }
                return Err(CodexError::new(
                    "codex_login_failed",
                    error.unwrap_or_else(|| "Вход в Codex не завершён".into()),
                ));
            }
        }
    }

    async fn connect(
        &self,
    ) -> Result<
        (
            tokio::process::Child,
            tokio::process::ChildStdin,
            Lines<BufReader<ChildStdout>>,
        ),
        CodexError,
    > {
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
        send(
            &mut stdin,
            &RpcRequest {
                method: "initialize",
                id: Some(1),
                params: InitializeParams {
                    client_info: ClientInfo {
                        name: "bookshelf",
                        title: "Bookshelf",
                        version: env!("CARGO_PKG_VERSION"),
                    },
                },
            },
        )
        .await?;
        let _: serde_json::Value = response(&mut lines, 1).await?;
        send(
            &mut stdin,
            &RpcRequest {
                method: "initialized",
                id: None,
                params: (),
            },
        )
        .await?;
        Ok((child, stdin, lines))
    }
}

async fn send(
    stdin: &mut tokio::process::ChildStdin,
    value: &impl Serialize,
) -> Result<(), CodexError> {
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

async fn response<T: DeserializeOwned>(
    lines: &mut Lines<BufReader<ChildStdout>>,
    id: u64,
) -> Result<T, CodexError> {
    loop {
        let line = lines
            .next_line()
            .await
            .map_err(|error| CodexError::new("codex_protocol_failed", error.to_string()))?
            .ok_or_else(|| CodexError::new("codex_crashed", "Codex завершился во время запуска"))?;
        let value: RpcResponse<T> = serde_json::from_str(&line).map_err(|_| {
            CodexError::new(
                "codex_protocol_incompatible",
                "Codex вернул неизвестный формат ответа",
            )
        })?;
        if value.id != id {
            continue;
        }
        if let Some(error) = value.error {
            let message = error.message;
            let code = if message.to_lowercase().contains("auth")
                || message.to_lowercase().contains("login")
            {
                "codex_login_required"
            } else {
                "codex_protocol_incompatible"
            };
            return Err(CodexError::new(code, message));
        }
        return value.result.ok_or_else(|| {
            CodexError::new(
                "codex_protocol_incompatible",
                "Codex вернул ответ без result",
            )
        });
    }
}

enum StreamMessage {
    Delta(String),
    Completed,
    Interrupted,
    Failed(String),
    Ignore,
}

fn stream_message(event: ServerEvent) -> StreamMessage {
    match event {
        ServerEvent::AgentMessageDelta { delta } => StreamMessage::Delta(delta),
        ServerEvent::TurnCompleted { turn } => match turn.status.as_str() {
            "completed" => StreamMessage::Completed,
            "interrupted" => StreamMessage::Interrupted,
            "failed" => StreamMessage::Failed(
                turn.error
                    .map(|error| error.message)
                    .unwrap_or_else(|| "Проверка Codex завершилась с ошибкой".into()),
            ),
            _ => StreamMessage::Ignore,
        },
        ServerEvent::Error { error } => StreamMessage::Failed(error.message),
        ServerEvent::LoginCompleted { .. } | ServerEvent::Other => StreamMessage::Ignore,
    }
}

fn codex_executable() -> io::Result<PathBuf> {
    #[cfg(debug_assertions)]
    if let Some(path) = env::var_os("BOOKSHELF_CODEX_SIDECAR") {
        return Ok(path.into());
    }
    if let Ok(current) = env::current_exe() {
        let name = if cfg!(windows) { "codex.exe" } else { "codex" };
        if let Some(parent) = current.parent() {
            let bundled = parent.join(name);
            if bundled.exists() {
                return Ok(bundled);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "bundled Codex sidecar is missing",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_maps_stream_deltas_and_terminal_states() {
        let delta: ServerEvent = serde_json::from_str(
            r#"{"method":"item/agentMessage/delta","params":{"delta":"Пробел"}}"#,
        )
        .unwrap();
        assert!(matches!(
            stream_message(delta),
            StreamMessage::Delta(value) if value == "Пробел"
        ));
        let complete: ServerEvent = serde_json::from_str(
            r#"{"method":"turn/completed","params":{"turn":{"status":"completed","error":null}}}"#,
        )
        .unwrap();
        assert!(matches!(stream_message(complete), StreamMessage::Completed));
        let interrupted: ServerEvent = serde_json::from_str(
            r#"{"method":"turn/completed","params":{"turn":{"status":"interrupted","error":null}}}"#,
        )
        .unwrap();
        assert!(matches!(
            stream_message(interrupted),
            StreamMessage::Interrupted
        ));
    }
}
