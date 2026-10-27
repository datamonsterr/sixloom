---
product: <Product name>
version: 0.1.0
origin: discovery        # discovery | reverse-engineered
updated: YYYY-MM-DD
---

# <Product name>

## Why

<Two or three sentences: the problem, who has it, and what this product
changes for them. Take it from discovery; do not invent it.>

## Users

| Actor | Who they are | What they need most |
|---|---|---|
| <Visitor> | <...> | <...> |
| <Member> | <...> | <...> |

## Features

<!-- One section per feature. IDs are permanent. Status: planned | changed | dropped -->

### F-001 <Feature name>

- **Priority**: must
- **Status**: planned
- **Stories**: US-01, US-02
- **What it does**: <One paragraph, user's point of view, no tech words.>

**Acceptance criteria**

- **F-001.AC1** Given <state>, when <action>, then <observable result>.
- **F-001.AC2** Given <state>, when <invalid action>, then <error the user sees>.
- **F-001.AC3** Given <the data is loading or failed>, then <what the user sees>.

### F-002 <Feature name>

- **Priority**: should
- **Status**: planned
- **Stories**: US-03
- **What it does**: <...>

**Acceptance criteria**

- **F-002.AC1** Given …, when …, then ….

## Key flows

<!-- The two to five journeys that cross features, as numbered steps with feature IDs. -->

1. **<First purchase>**: F-001 → F-003 → F-004

## Rules and limits

<!-- Business rules that span features: limits, permissions, money, time zones, retention. -->

- R-01 <A member can have at most 5 active projects.>

## Out of scope (v1)

- <Thing people might expect but will not get, and why.>

## Open decisions

| ID | Question | Recommended | Status |
|---|---|---|---|
| Q-01 | <...> | <...> | open / decided / deferred |
