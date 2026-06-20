# Rules Reference (WIP)

> **Status: Work in Progress** -- rule codes, behaviour, and configuration options may change before 1.0.

---

## Overview

Every rule has a code (`SLxxx`), a default severity, and can be individually
configured in `pyproject.toml` (see [Configuration](#configuration)).

| Code | Default | Description | Status |
|--------|---------|----------------------------------------------------------|--------|
| SL001 | error | No extra positional arguments after the event string | done |
| SL002 | error | Do not use f-strings as the event argument | done |
| SL003 | error | Do not use `%`-formatting in the event argument | done |
| SL004 | error | Do not use `.format()` in the event argument | done |
| SL005 | error | `log.exception()` only inside `except` blocks | done |
| SL006 | error | Prefer `log.exception()` over `log.error()` in `except` | done |
| SL007 | warning | No logging above `debug` inside loop bodies | done |
| SL008 | error | Event string must match the configured case style | done |
| SL009 | warning | Event string exceeds maximum length | done |

---

## Logger Detection

`structloglint` uses a **naming convention heuristic** to identify structlog
logger calls — similar to the approach used by
[Ruff](https://docs.astral.sh/ruff/). It does **not** perform full semantic
analysis such as import tracing, type inference, or assignment tracking.

### Prerequisites

A file is only checked if it contains a structlog import at the top level:

```python
import structlog
# or
from structlog import get_logger
```

### What is detected

A call `X.method(...)` is treated as a structlog log call when:

1. `method` is a known log level: `trace`, `debug`, `info`, `warning`,
   `error`, `critical`, or `exception`
2. `X` follows a logger naming convention:
   - Starts with `log` (e.g. `log`, `logger`, `logging`, `log_ctx`)
   - Starts with `LOG` (e.g. `LOG`, `LOGGER`)
   - Ends with `logger` (e.g. `my_logger`, `app_logger`)
   - Ends with `LOGGER` (e.g. `MY_LOGGER`)

Attribute chains are also supported — `self.logger.info(...)` matches
because the last attribute before the method (`logger`) satisfies the
naming convention.

### Detected patterns

```python
# All detected
log.info("event")
logger.info("event")
self.log.info("event")
self.logger.info("event")
cls.logger.info("event")
app.logger.info("event")
my_logger.info("event")

# NOT detected — name doesn't match convention
svc.info("event")
ctx.info("event")
self.helper.info("event")
```

### Recommended naming

Use `log` or `logger` as your logger variable name:

```python
import structlog

log = structlog.get_logger()
```

### Bound loggers

structlog's `.bind()` returns a new logger instance. Rebind to the same
variable name so the linter continues to detect calls:

```python
log = structlog.get_logger()

def handle_request(request_id: str, user_id: str):
    log = log.bind(request_id=request_id, user_id=user_id)
    log.info("request_started")
```

Avoid inventing new variable names that don't follow the convention:

```python
# Not recommended — linter won't detect calls on "ctx"
ctx = log.bind(request_id=request_id)
ctx.info("request_started")  # invisible to the linter
```

---

## SL001 -- No extra positional arguments

Only one positional argument (the event string) is allowed. All context must
be passed as keyword arguments.

```python
# bad
log.info("user_logged_in", user_id)
log.info("payment_processed", user_id, order_id, 4999)
log.warning("rate_limit_exceeded", user_id, limit=100)

# good
log.info("user_logged_in", user_id=user_id)
log.info("payment_processed", user_id=user_id, order_id=order_id, amount=4999)
```

---

## SL002 -- No f-string events

The event argument must be a constant string. f-strings bake variable data
into the event name, making it impossible to filter or aggregate by event.

```python
# bad
log.info(f"user {user_id} logged in")
log.warning(f"rate limit exceeded")

# good
log.info("user_logged_in", user_id=user_id)
```

---

## SL003 -- No `%`-formatting in events

Same rationale as SL002. Old-style `%` formatting in the event string produces
dynamic event names.

```python
# bad
log.info("user %s signed up" % username)

# good
log.info("user_signed_up", username=username)
```

---

## SL004 -- No `.format()` in events

Same rationale as SL002/SL003. `.format()` on the event string produces
dynamic event names.

```python
# bad
log.info("subscription cancelled for {}".format(user_id))

# good
log.info("subscription_cancelled", user_id=user_id)
```

---

## SL005 -- `exception()` only inside `except`

`log.exception()` captures the current exception's traceback via
`sys.exc_info()`. Outside an `except` block there is no active exception, so
the call is a bug.

```python
# bad
def notify(user_id):
    log.exception("notification_failed", user_id=user_id)

# good
def notify(user_id):
    try:
        _send(user_id)
    except Exception:
        log.exception("notification_failed", user_id=user_id)
        raise
```

---

## SL006 -- Prefer `exception()` over `error()` in `except` (wip)

Inside an `except` block, using `log.error()` silently discards the traceback.
Use `log.exception()` instead, or pass `exc_info=True` explicitly if
`error()` is intentional.

```python
# bad
try:
    charge(order_id)
except TimeoutError as e:
    log.error("charge_timed_out", order_id=order_id, error=str(e))

# good -- use exception()
except TimeoutError:
    log.exception("charge_timed_out", order_id=order_id)

# good -- explicit opt-out
except TimeoutError:
    log.error("charge_timed_out", order_id=order_id, exc_info=True)
```

---

## SL007 -- No logging above `debug` inside loops (wip)

Log calls at `info` level or higher inside `for` / `while` / `async for`
loop bodies can produce excessive output. Only `log.debug()` is allowed
inside loops.

```python
# bad
for product in products:
    log.info("product_imported", product_id=product["id"])

# good -- use debug, or log a summary after the loop
for product in products:
    log.debug("product_imported", product_id=product["id"])
log.info("product_import_complete", count=len(products))
```

NOTE: Only one level deep inside loop bodies is checked.

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `loop-log-level` | string | `"info"` | Minimum level that triggers a flag inside a loop body. Logs **at or above** this level are flagged. |

---

## SL008 -- Event string case style (wip)

Event strings should follow a consistent case convention. The default is
`snake_case`, matching structlog best practices. The case style is
configurable.

```python
# bad (default: snake_case)
log.info("ProfileUpdated", user_id=user_id)
log.info("profileUpdated", user_id=user_id)
log.info("profile updated successfully", user_id=user_id)
log.info("profile-updated", user_id=user_id)
log.info("PROFILE_UPDATED", user_id=user_id)

# good
log.info("profile_updated", user_id=user_id)
```

### Auto-fix

This rule supports `--fix SL008`. When enabled, all event strings are
rewritten in-place to match the configured `event-case-style`, regardless of
their current case convention.

```bash
# Convert all events to snake_case (default)
structloglint --file path/to/file.py --fix SL008

# Convert all events to camelCase
structloglint --file path/to/file.py --fix SL008 --event-case-style camelCase
```

```python
# before (mixed case)
log.info("ProfileUpdated", user_id=user_id)
log.warning("order-cancelled", order_id=order_id)
log.debug("PAYMENT_RECEIVED", amount=amount)

# after (--fix SL008, default snake_case)
log.info("profile_updated", user_id=user_id)
log.warning("order_cancelled", order_id=order_id)
log.debug("payment_received", amount=amount)
```

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `event-case-style` | string | `"snake_case"` | Required case style for event strings. |

Supported values for `event-case-style`:

| Value | Example |
|-------|---------|
| `snake_case` | `user_logged_in` |
| `kebab-case` | `user-logged-in` |
| `camelCase` | `userLoggedIn` |
| `PascalCase` | `UserLoggedIn` |
| `SCREAMING_SNAKE_CASE` | `USER_LOGGED_IN` |

---

## SL009 -- Event string maximum length (wip)

Event strings that are too long are hard to read and indicate that
the event is a sentence rather than a machine-friendly identifier.

```python
# bad (assuming max-event-length = 30)
log.info("the user has successfully logged into the system and was redirected to the dashboard")

# good
log.info("user_logged_in", redirect="dashboard")
```

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max-event-length` | integer | `30` | Maximum number of characters allowed in the event string. |

---

## Future Rules

The following rules are under consideration for future versions:

| Code | Description | Notes |
|--------|----------------------------------------------------------|-------|
| SL010 | Event string should use past tense | Needs reliable verb tense detection; current NLP approaches (stemming + word lists) produce too many false positives on noun-phrase events. |

---

## Configuration

`structloglint` reads configuration from `pyproject.toml` under the
`[tool.structloglint]` key, following the same conventions as
[ruff](https://docs.astral.sh/ruff/) and [ty](https://docs.astral.sh/ty/).

### Minimal example

```toml
[tool.structloglint]
include = ["src/**/*.py", "app/**/*.py"]
exclude = ["tests/**", "migrations/**"]

select = ["SL001", "SL002", "SL003", "SL004", "SL005", "SL006", "SL007", "SL008", "SL009"]
ignore = ["SL007"]
max-event-length = 40
event-case-style = "snake_case"
loop-log-level = "info"

[tool.structloglint.rules]
SL006 = "error"      # promote to error
SL007 = "off"        # disable loop check
```

### Top-level options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `include` | list of globs | `["**/*.py"]` | File patterns to check. |
| `exclude` | list of globs | `[]` | File patterns to skip (takes precedence over `include`). |
| `include-files` | list of paths | `[]` | Explicit file paths to check. |
| `exclude-files` | list of paths | `[]` | Explicit file paths to skip. |
| `select` | list of rules | (all) | Only run these rules. |
| `ignore` | list of rules | (none) | Skip these rules (applied after `select`). |
| `max-event-length` | integer | `30` | Maximum event string length (SL009). |
| `event-case-style` | string | `"snake_case"` | Required event case style (SL008). |
| `loop-log-level` | string | `"info"` | Minimum level that triggers a flag inside loops (SL007). |

### Per-rule severity overrides (`[tool.structloglint.rules]`)

Every rule code can be set to one of:

| Value | Meaning |
|-------|---------|
| `"error"` | Report as an error (non-zero exit code). |
| `"warning"` | Report as a warning. |
| `"off"` | Disable the rule entirely. |

```toml
[tool.structloglint.rules]
SL001 = "error"
SL008 = "warning"
```

### Installation

`structloglint` will be published as a pip-installable package:

```
pip install structloglint
```

The binary is written in Rust and distributed as a Python wheel (via
[PyO3](https://pyo3.rs/) / [maturin](https://www.maturin.rs/)), so no Rust
toolchain is required at install time.
