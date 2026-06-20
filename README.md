# structlog-linter

A fast, opinionated linter for [structlog](https://www.structlog.org/) log calls in Python. Written in Rust.

> **Status: Early development (v0.1.0)** -- not yet published as a pip-installable package.

## Current Usage

```bash
cargo build --release
./target/release/structlog_linter --path src/ --verbose
```

See [RULES.md](RULES.md) for the full list of rules (SL001--SL009) and planned configuration options.

## Roadmap (MoSCoW)

### Done

| Area | Detail |
|------|--------|
| Linting rules | SL001--SL009 fully implemented with unit tests |
| AST walker | Recursive Python AST traversal via `rustpython-parser` with parent context tracking (19 block types) |
| Logger detection | Naming-convention heuristic (matches `log`, `logger`, `LOG`, `*_logger`, etc.) |
| CLI | `--path`, `--verbose`, `--output-format full\|concise` via `clap` |
| Output | Colored diagnostics with source context, gutter markers, and underline spans |
| Fix data model | `Fix` struct exists, SL008 generates fix suggestions |
| Test data | Per-rule Python fixtures (`test_data/SL00x.py`) + comprehensive e-commerce example |

### Must Have

| Area | Detail | Status |
|------|--------|--------|
| Python packaging | PyO3 + maturin build so `pip install structlog-linter` works | not started |
| `pyproject.toml` config | Parse `[tool.structlog-linter]` for include/exclude, case style, max length, per-rule severity | not started (values hardcoded) |
| `--fix` flag | Apply auto-fixes in-place (SL008 fix model already exists) | not started |
| CLI parity | `--file` single-file mode, `--event-case-style`, `--loop-log-level` flags | not started |
| Pre-commit hook | `.pre-commit-hooks.yaml` so repos can add the linter to `.pre-commit-config.yaml` | not started |
| CI/CD | GitHub Actions: lint, test, build on push/PR | not started |
| Release management | `release-please` for automated changelogs, version bumps, and GitHub releases | not started |
| Cross-platform builds | CI matrix for Linux, macOS, Windows (x86_64 + aarch64) | not started |
| Binary distribution | Publish wheels per platform to PyPI via maturin | not started |
| Exit codes | Non-zero exit on findings for CI gating | not started |
| Integration tests | End-to-end tests that invoke the binary against `test_data/` and assert output/exit code | not started |
| Inline suppression | `# noqa: SL001` style comments to suppress per-line | not started |

### Should Have

| Area | Detail | Status |
|------|--------|--------|
| CHANGELOG | Auto-generated via release-please | not started |
| SARIF / JSON output | Machine-readable output for CI integrations (GitHub code scanning, etc.) | not started |
| `--diff` mode | Show fixes as unified diffs without applying | not started |
| SL010 rule | Event string should use past tense | not started |
| Benchmarks | Performance regression tracking in CI | not started |

### Could Have

| Area | Detail |
|------|--------|
| GitHub Action | Marketplace action for direct use in workflows |
| Editor integration | VSCode extension / LSP server |
| Custom rule plugins | User-defined rules via config or scripting |
| Watch mode | Re-lint on file change |

### Won't Have (for now)

| Area | Reason |
|------|--------|
| Full type inference | Would require a Python type checker; naming heuristic is sufficient |
| Auto-fix for all rules | Some rules (SL005, SL007) require structural refactoring that can't be automated safely |
| flake8 plugin mode | Targeting standalone tool distribution like ruff instead |

## License

MIT
