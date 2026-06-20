# Contributing

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [pre-commit](https://pre-commit.com/)
- [maturin](https://www.maturin.rs/) (for Python packaging)

## Setup

```bash
git clone https://github.com/maeldorne/structloglint.git
cd structloglint
make install-hooks
```

## Development

```bash
make check       # fmt + clippy + test (run before pushing)
make build       # release build
make dev         # install into active venv via maturin
```

## Commit conventions

This project uses [Conventional Commits](https://www.conventionalcommits.org/). All commit messages are validated by a pre-commit hook.

| Prefix | Purpose |
|--------|---------|
| `feat:` | New feature (minor version bump) |
| `fix:` | Bug fix (patch version bump) |
| `perf:` | Performance improvement |
| `docs:` | Documentation only |
| `chore:` | Maintenance, dependencies |
| `ci:` | CI/CD changes |
| `refactor:` | Code change that neither fixes a bug nor adds a feature |
| `test:` | Adding or updating tests |
| `build:` | Build system changes |

Breaking changes: add `!` after the prefix (e.g. `feat!: remove --verbose flag`) or include `BREAKING CHANGE:` in the commit body.

## Testing

```bash
make test
```

Tests live alongside the source code in `#[cfg(test)]` modules. Test fixtures are in `test_data/`.

## Adding a new rule

1. Create `src/rules/slXXX.rs` with a `check_slXXX` function
2. Register it in `src/rules/mod.rs`
3. Add a test fixture in `test_data/SLXXX.py`
4. Add documentation in `RULES.md`

## Releases

Releases are automated via [release-please](https://github.com/googleapis/release-please). When commits land on `main`, release-please opens a PR with version bumps and a changelog. Merging that PR triggers a GitHub release and publishes wheels to PyPI.
