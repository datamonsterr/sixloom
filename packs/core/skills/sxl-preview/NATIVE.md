# TUI, CLI, mobile and desktop interfaces

These kinds have no browser network layer to intercept, so the mock sits
**behind the client interface**. Paths are relative to the interface's `path`
from `sixloom layout`. Record the real module paths with
`sixloom layout interface add … --api-client … --mocks …`.

## The client seam

Define the client as an interface (trait, protocol, Go interface, TS type)
with one method per operation the screens need. Provide two
implementations: `HttpClient` (talks to the backend, written in the build
loom) and `MockClient` (in-memory fixtures plus a scenario). Choose one at
startup:

```rust
// Ratatui example: src/client.rs
pub trait Api {
    fn list_tasks(&self) -> Result<Vec<Task>, ApiError>;
    fn create_task(&self, title: &str) -> Result<Task, ApiError>;
}

pub fn from_env() -> Box<dyn Api> {
    match std::env::var("SXL_API_MODE").as_deref() {
        Ok("mock") => Box::new(mocks::MockClient::new(mocks::Scenario::from_env())),
        _ => Box::new(HttpClient::new(std::env::var("API_URL").unwrap_or_default())),
    }
}
```

```python
# Textual example: tasklet/client.py
class Api(Protocol):
    async def list_tasks(self) -> list[Task]: ...
    async def create_task(self, title: str) -> Task: ...

def from_env() -> Api:
    return MockClient(Scenario.from_env()) if os.getenv("SXL_API_MODE") == "mock" else HttpClient(os.environ["API_URL"])
```

- **Scenarios** are read from `SXL_SCENARIO` (`ready`, `empty`, `error`,
  `slow`, `offline`, …), or from a hidden `--sxl-scenario` flag for CLIs.
- **Latency**: the mock sleeps a realistic amount (async, never blocking the
  render loop), so spinners and loading rows actually show.
- **Errors** use the same envelope the API will use: `{ code, message, fields? }`.
- **Stateful**: create, update and delete change the in-memory fixtures, so
  flows feel real.
- **Fixtures** live as JSON or YAML files under `<mocks>/fixtures/`. The
  API loom validates them against the contract, the same way it validates web
  mock responses.

## Running each state (what PREVIEW.md lists)

| Kind | Preview command | State |
|---|---|---|
| TUI | `SXL_API_MODE=mock SXL_SCENARIO=empty cargo run -p tui` | which keys reach the screen |
| CLI | `SXL_API_MODE=mock tasklet list --sxl-scenario error` | expected output and exit code |
| mobile | `SXL_API_MODE=mock npx expo start` + an in-app dev menu to pick the scenario | screen route |
| desktop | `SXL_API_MODE=mock npm run tauri dev` | screen route |

## Handles and trace

- **TUI**: give each widget or focusable area a stable name (for example a
  `const` key or ID used in tests). Put the trace comment on the function that
  renders or handles it: `// sxl: F-001.AC2`.
- **CLI**: commands and flags are the handles. Put the trace comment on the
  command handler.
- **mobile/desktop**: use accessibility IDs (`testID`) as handles and the
  `sxl:` comment on the screen component.

Comments are never compiled into the binary, so nothing needs stripping.
`sixloom trace` reads both `data-sxl` and `sxl:` comments in every interface
folder.

## Smoke check

- **TUI**: render every screen × state in the framework's test harness and
  save the text buffer: Ratatui `TestBackend`, Textual `Pilot` /
  `app.run_test()`, Bubble Tea `teatest`, Ink `ink-testing-library`.
- **CLI**: run every command × scenario and save stdout and the exit code.

Save the results to `<path>/.preview-shots/` (gitignored). The validate loom
compares the real app against them.
