# Project Rules

## Review

After completing changes with `/dev`, follow this review cycle:

1. `/rev` to review changes
2. Fix Critical / Warning / Suggestion via `/dev` (repeat 1-2 until clean)
3. `/rev` to catch missing doc updates, fix via `/dev`
4. Report subjective or ambiguous items for human decision

- A single review cycle at the end is sufficient for multi-file changes

## Documentation

- Keep `README.md` and `docs/*.md` up to date whenever code changes affect documented behavior, architecture, or setup instructions

## Frontend

- After modifying any file under `client/`, always run `cd client && bunx vite build` to verify the build succeeds before considering the task complete
- Review (`/rev`) must include a build check when frontend files are changed
