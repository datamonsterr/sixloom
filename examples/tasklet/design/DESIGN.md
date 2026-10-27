# Design system

## Direction

A quiet paper notebook: warm off-white, ink-dark text, one terracotta accent for the only action.

## Tokens

| Token | Value | Use |
|---|---|---|
| `--color-bg` | `#faf7f2` | page |
| `--color-fg` | `#1d1b18` | text |
| `--color-muted` | `#7a746b` | secondary |
| `--color-accent` | `#c4532d` | Add button, focus |
| `--color-danger` | `#b42318` | errors |
| `--color-border` | `#e4ddd2` | rows, input |
| `--font-body` | "Iowan Old Style", Georgia, serif · 16/24 | body |
| `--radius` | 6px | inputs, buttons |

## State patterns

- **Loading**: 3 skeleton rows, 44px tall, shimmer.
- **Empty**: one sentence, input focused.
- **Error**: message + Retry; keep the typed title.
