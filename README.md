# structloglint

A fast, opinionated linter for [structlog](https://www.structlog.org/) log calls in Python. Written in Rust.

> **Status: Early development** — [GitHub](https://github.com/maxilar20/structloglint)

## Installation

```bash
pip install structloglint
```

## Usage

```bash
structloglint --path src/
structloglint --path . --select SL001,SL002 --ignore SL007 --output-format concise
structloglint --path . --max-event-length 40 --event-case-style camel_case
```

See [RULES.md](RULES.md) for the full list of rules (SL001--SL009) and configuration options.

## Configuration

Add a `[tool.structloglint]` section to your `pyproject.toml`:

```toml
[tool.structloglint]
# Event string rules
event-case-style = "snake_case"
max-event-length = 30

# Loop logging
loop-log-level = "info"

# Rule selection (omit to run all)
select = ["SL001", "SL002", "SL003", "SL004", "SL005", "SL006", "SL007", "SL008", "SL009"]
ignore = []

# File discovery
include = ["src/**/*.py", "app/**/*.py"]
exclude = ["tests/**", "migrations/**"]
extend-exclude = ["generated/**"]

# Import detection
check-imports = true

# Per-rule severity overrides
[tool.structloglint.rules]
SL007 = "off"       # disable loop check
SL009 = "error"     # promote to error
```

Alternatively, create a standalone `structloglint.toml` in your project root (takes precedence over `pyproject.toml`). In standalone format, omit the `[tool.structloglint]` table wrapper:

```toml
# structloglint.toml (standalone)
event-case-style = "camel_case"
max-event-length = 50
loop-log-level = "info"
select = ["SL001", "SL002", "SL003", "SL004", "SL005", "SL006", "SL007", "SL008", "SL009"]
ignore = ["SL007"]
exclude = ["tests/**"]
check-imports = true

[rules]
SL007 = "off"
```

See [RULES.md](RULES.md#configuration) for all configuration options and rule descriptions.

See [ROADMAP.md](ROADMAP.md) for the project plan.

## License

MIT
