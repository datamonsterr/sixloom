---
product: Jot
version: 0.1.0
origin: discovery
updated: 2026-10-28
---

# Jot

## Why

Terminal users want to see and add notes without leaving the shell.

## Users

| Actor | Who they are | What they need most |
|---|---|---|
| Developer | Lives in the terminal | Fast notes |

## Features

### F-001 See notes

- **Priority**: must
- **Status**: planned
- **Stories**: US-01
- **What it does**: Lists notes, newest first.

**Acceptance criteria**

- **F-001.AC1** Given I have notes, when I open Jot, then I see them newest first.
- **F-001.AC2** Given I have no notes, when I open Jot, then I see "No notes yet".

## Out of scope (v1)

- Editing notes.

## Open decisions

| ID | Question | Recommended | Status |
|---|---|---|---|
| Q-01 | Sync? | No | decided |
