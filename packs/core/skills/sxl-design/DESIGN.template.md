# Design system

<!-- Tokens are the contract between design and code. The preview reads them; never hard-code values. -->

## Direction

<Two sentences: the subject's world, the mood, and the one thing that should make this look like no other product.>

## Tokens

### Color

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-bg` | `#ffffff` | `#0b0b0c` | page background |
| `--color-fg` | `#111113` | `#ededef` | body text |
| `--color-muted` | | | secondary text |
| `--color-accent` | | | primary actions, focus ring |
| `--color-danger` | | | destructive, errors |
| `--color-border` | | | dividers, inputs |

### Type

| Token | Family | Size / line | Weight | Use |
|---|---|---|---|---|
| `--font-display` | | 32 / 40 | 600 | page titles |
| `--font-body` | | 15 / 22 | 400 | body |
| `--font-mono` | | 13 / 20 | 400 | code, IDs |

### Space, radius, elevation, motion

- Space scale: 4, 8, 12, 16, 24, 32, 48
- Radius: `--radius-sm` 4, `--radius-md` 8
- Elevation: <none | border-only | 2 shadow levels>
- Motion: 150ms ease-out for state changes; respect `prefers-reduced-motion`

## Components

| Component | Variants | Rules |
|---|---|---|
| Button | primary, secondary, ghost, danger | one primary per view |
| Input | default, error, disabled | error text below, never placeholder-only labels |

## State patterns

- **Loading**: skeleton blocks matching final layout; shimmer ≤ 1.2s loop; never block the whole page for a partial load.
- **Empty**: what this is, why it is empty, one action.
- **Error**: plain-language message, what to do, Retry when retryable; keep user input.
- **Submitting**: disable the submit control, keep the label, show progress inline.

## Accessibility

Contrast ≥ 4.5:1 for text; visible focus; every control reachable by keyboard; labels on all inputs.
