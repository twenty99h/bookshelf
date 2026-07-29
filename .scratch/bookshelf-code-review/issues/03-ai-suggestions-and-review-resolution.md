# 03 — Review AI suggestions and review resolution

Type: task
Status: resolved

## Review findings

- Confirming a suggested topic creates it but does not assign it to the reviewed idea.
- «Разобрать позже» contributes to aggregate debt but exposes no visible resolution action.
- A finalized response can remain in snapshots/transfer archives despite being described as temporary.

## Acceptance

- [x] Topic confirmation atomically creates and assigns the topic.
- [x] Pending responses are directly reopenable and resolvable.
- [x] Snapshots and transfer archives omit full transient AI responses.

## Answer

Topic confirmation is one domain action, pending review cards reopen the response, and every snapshot/archive is derived from a transient-response-scrubbed state. Regression tests cover assignment and scrubbing.
