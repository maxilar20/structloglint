# Roadmap

> MoSCoW priority. Checkmarks indicate shipped features.

## Must Have

- [x] SL001--SL009 linting rules with unit tests
- [x] AST walker with parent context tracking (19 block types)
- [x] Logger detection via naming-convention heuristic
- [x] CLI: `--path`, `--verbose`, `--output-format full|concise`
- [x] CLI config overrides: `--event-case-style`, `--max-event-length`, `--loop-log-level`
- [x] Rule selection: `--select` / `--ignore` CLI flags + config keys
- [x] Per-rule severity overrides (`off` / `warning` / `error`)
- [x] Colored diagnostics with source context and gutter markers
- [x] Configuration: `pyproject.toml` + standalone `structloglint.toml` with directory-tree discovery
- [x] File filtering: `exclude`, `extend-exclude`, `include` glob patterns
- [x] Default excludes for venv, node_modules, caches, etc.
- [x] Non-zero exit codes for CI gating
- [ ] `--fix` flag (auto-fix in-place; SL008 fix model already exists)
- [ ] Inline suppression (`# noqa: SL001` comments)

## Should Have

- [x] Python packaging via maturin (pip install)
- [x] Pre-commit hook
- [x] CI/CD: GitHub Actions for fmt, clippy, test
- [x] Cross-platform builds (Linux, macOS, Windows)
- [x] Binary distribution (platform wheels on PyPI)
- [x] Release management via release-please
- [x] Integration tests with fixture packages
- [ ] `--diff` mode (show fixes without applying)
- [ ] JSON / SARIF output formats
- [ ] `--output-format github` for PR annotations
- [ ] SL010 rule (event past tense)
- [ ] Benchmarks

## Could Have

- [ ] GitHub Action marketplace listing
- [ ] Editor integration / LSP server
- [ ] Watch mode

## Won't Have (for now)

- Custom rule plugins
- Full type inference (naming heuristic is sufficient)
- Auto-fix for all rules (some require structural refactoring)
- flake8 plugin mode (targeting standalone distribution)
