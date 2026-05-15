# Contributing to PennyKite

Thanks for your interest. PennyKite is an open-source MIT project and we
welcome contributions.

## Ground rules

- All commits, code, documentation and communication in **English**.
- Follow [conventional commits](https://www.conventionalcommits.org/):
  `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`, `ci:`.
- Push directly to `main`. No PR ceremony for core contributors.
- External contributors: open an issue before sending a non-trivial PR.

## Development setup

```bash
git clone https://github.com/manuelpenazuniga/PennyKite.git
cd PennyKite
cp .env.example .env   # fill in required env vars
```

## Before pushing

```bash
cargo test --workspace
forge test -vvv           # from contracts/
npm run build             # from dashboard/
```

## Project structure

See `README.md` § "Project structure" for the full tree. Do not invent new
top-level directories.

## Where to find work

The atomic execution plan lives in [`backlog.yaml`](./backlog.yaml). Filter
for `status: pending` and pick the highest-priority task whose dependencies
are all `done`.

## Licence

By contributing you agree that your work is licensed under the MIT licence
of this project.
