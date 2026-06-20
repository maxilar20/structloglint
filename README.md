# structlog-linter

A fast, opinionated linter for [structlog](https://www.structlog.org/) log calls in Python. Written in Rust.

## Installation

```
pip install structlog-linter
```

## Usage

```
structlog-linter --file path/to/file.py
```

## Configuration

Add a `[tool.structlog-linter]` section to your `pyproject.toml`:

```toml
[tool.structlog-linter]
include = ["src/**/*.py"]
exclude = ["tests/**"]
event-case-style = "snake_case"
max-event-length = 50

[tool.structlog-linter.rules]
SL007 = "error"
SL010 = "warning"
```

See [RULES.md](RULES.md) for the full list of rules and configuration options.

## License

MIT
