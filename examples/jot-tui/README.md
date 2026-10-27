# Jot: sixloom with a TUI instead of a web app

A small Ratatui terminal app, set up with

```bash
sixloom init --interface tui:ratatui:crates/tui --docs starlight --design none
sixloom layout interface add tui:ratatui:crates/tui --api-client src/client.rs --mocks src/mocks.rs
sixloom layout locked add crates/tui/tests
sixloom layout pattern local-only
```

It shows how the layout follows the stack:

- **`sixloom.toml`** records the interface (`kind = "tui"`, folder `crates/tui`,
  client `src/client.rs`, mocks `src/mocks.rs`), the `local-only` pattern (no
  services), and `crates/tui/tests` as a locked test folder.
- **The preview** runs on a mock client behind the `Api` trait:
  `SXL_API_MODE=mock SXL_SCENARIO=empty cargo run -p jot-tui`
  (see [`crates/tui/PREVIEW.md`](crates/tui/PREVIEW.md)).
- **Trace** uses a `// sxl: F-001.AC1, F-001.AC2` comment on the render function
  (nothing in the binary), and the acceptance tests name their AC:
  `fn f001_ac1_newest_first()`.
- **Locked tests** are Rust: `sixloom tests verify` fails if an `assert_eq!` is
  weakened or a test gets `#[ignore]`.

Run `cargo test` for the acceptance tests, and `sixloom update` to install the
skills, which are not checked in here.
