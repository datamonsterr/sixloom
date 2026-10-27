# Preview

Run `npm run preview` in `apps/web` (mock mode, http://localhost:5173).

| Screen | State | URL |
|---|---|---|
| S-01 | loading → ready | http://localhost:5173/?sxl=ready |
| S-01 | loading (3s) | http://localhost:5173/?sxl=slow |
| S-01 | empty | http://localhost:5173/?sxl=empty |
| S-01 | error | http://localhost:5173/?sxl=error |
| S-01 | offline | http://localhost:5173/?sxl=offline |
| S-01 | validation-error | http://localhost:5173/?sxl=empty → press Add with an empty title |
