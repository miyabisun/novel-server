# Project Rules

## Review

- After updating files, always call `/rev` to get a review before considering the task complete
- When multiple files are changed, a single review at the end is sufficient

## Documentation

- Keep `README.md` and `docs/*.md` up to date whenever code changes affect documented behavior, architecture, or setup instructions

## Frontend

- After modifying any file under `client/`, always run `cd client && npx vite build` to verify the build succeeds before considering the task complete
- Review (`/rev`) must include a build check when frontend files are changed
