# structloglint

A fast, opinionated linter for [structlog](https://www.structlog.org/) log calls in Python. Written in Rust.

> **Status: Early development (v0.1.1)**

## Installation

```bash
uv pip install structloglint
```

### From source

```bash
uv venv
source .venv/bin/activate
uv pip install maturin
maturin develop --release
```

## Usage

```bash
structloglint --path src/ --verbose
```

See [RULES.md](RULES.md) for the full list of rules (SL001--SL009) and configuration options.

## Configuration

Add a `[tool.structloglint]` section to your `pyproject.toml`:

```toml
[tool.structloglint]
event-case-style = "snake_case"
max-event-length = 30
loop-log-level = "info"
```

Alternatively, create a standalone `structloglint.toml` in your project root (takes precedence over `pyproject.toml`).

See [RULES.md](RULES.md#configuration) for all configuration options.

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
| Configuration | `pyproject.toml` (`[tool.structloglint]`) and standalone `structloglint.toml` with directory-tree discovery |
| Integration tests | 14 integration tests using fixture Python packages with config variations |
| Python packaging | maturin build so `uv pip install structloglint` works |
| Pre-commit hook | `.pre-commit-hooks.yaml` for use in `.pre-commit-config.yaml` |
| CI/CD | GitHub Actions: fmt, clippy, test on push/PR |
| Release management | release-please for automated changelogs, version bumps, and GitHub releases |
| Cross-platform builds | CI matrix for Linux (x86_64, musl, aarch64), macOS (x86_64, aarch64), Windows (x86_64) |
| Binary distribution | Publish wheels per platform to PyPI via maturin |
| Exit codes | Non-zero exit on findings for CI gating |

### Must Have

| Area | Detail | Status |
|------|--------|--------|
| `--fix` flag | Apply auto-fixes in-place (SL008 fix model already exists) | not started |
| CLI config overrides | `--event-case-style`, `--max-event-length`, `--loop-log-level` flags | not started |
| Rule selection | `--select` / `--ignore` CLI flags + `select` / `ignore` config keys | not started |
| Inline suppression | `# noqa: SL001` style comments to suppress per-line | not started |

### Should Have

| Area | Detail | Status |
|------|--------|--------|
| JSON / SARIF output | Machine-readable output for CI integrations (GitHub code scanning, etc.) | not started |
| GitHub Actions output | `--output-format github` for inline PR annotations | not started |
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
