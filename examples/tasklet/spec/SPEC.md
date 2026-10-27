---
product: Tasklet
version: 0.1.0
origin: discovery
updated: 2026-10-27
---

# Tasklet

## Why

People juggling small personal to-dos lose them in chat threads and notes. Tasklet is one list that loads fast and takes a new task in a second.

## Users

| Actor | Who they are | What they need most |
|---|---|---|
| Member | Anyone with the app open | See tasks, add one quickly |

## Features

### F-001 See my tasks

- **Priority**: must
- **Status**: planned
- **Stories**: US-01
- **What it does**: Shows all tasks, newest first.

**Acceptance criteria**

- **F-001.AC1** Given I have tasks, when I open Tasklet, then I see them newest first.
- **F-001.AC2** Given I have no tasks, when I open Tasklet, then I see "All clear" and a way to add one.
- **F-001.AC3** Given the tasks fail to load, when I open Tasklet, then I see an error with a Retry button that reloads them.

### F-002 Add a task

- **Priority**: must
- **Status**: planned
- **Stories**: US-02
- **What it does**: Adds a task from one text box.

**Acceptance criteria**

- **F-002.AC1** Given I type a title and press Add, then the task appears at the top of the list and the box clears.
- **F-002.AC2** Given the title is empty, when I press Add, then I see "Title is required" and nothing is added.

## Key flows

1. **First task**: F-001 (empty) → F-002 → F-001

## Rules and limits

- R-01 A title is 1–120 characters.

## Out of scope (v1)

- Editing, completing or deleting tasks, accounts.

## Open decisions

| ID | Question | Recommended | Status |
|---|---|---|---|
| Q-01 | Persist tasks per browser or per account? | Per server, single list | decided |
