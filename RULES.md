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
| SL006 | warning | Prefer `log.exception()` over `log.error()` in `except` | done |
| SL007 | warning | No logging above `debug` inside loop bodies | done |
| SL008 | error | Event string must match the configured case style | wip |
| SL009 | warning | Event string exceeds maximum length | wip |
| SL010 | off | Event string should use past tense | wip |

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
| `loop-log-level` | string | `"debug"` | Maximum level allowed inside a loop body. Logs **above** this level are flagged. |

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
structlog-linter --file path/to/file.py --fix SL008

# Convert all events to camelCase
structlog-linter --file path/to/file.py --fix SL008 --event-case-style camelCase
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
# bad (assuming max-event-length = 50)
log.info("the user has successfully logged into the system and was redirected to the dashboard")

# good
log.info("user_logged_in", redirect="dashboard")
```

### Configuration

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max-event-length` | integer | `50` | Maximum number of characters allowed in the event string. |

---

## SL010 -- Past tense event names (wip)

Event names should describe something that **has happened** (past tense)
rather than something that **is happening** (present/progressive). This rule
is **off by default** and must be explicitly enabled.

The check is compatible with all supported `event-case-style` values; it
splits the event into words according to the active case style before
checking tense.

```python
# bad
log.info("user_logging_in", user_id=user_id)
log.info("processing_payment", order_id=order_id)
log.info("send_email", user_id=user_id)

# good
log.info("user_logged_in", user_id=user_id)
log.info("payment_processed", order_id=order_id)
log.info("email_sent", user_id=user_id)
```

---

## Configuration

`structlog-linter` reads configuration from `pyproject.toml` under the
`[tool.structlog-linter]` key, following the same conventions as
[ruff](https://docs.astral.sh/ruff/) and [ty](https://docs.astral.sh/ty/).

### Minimal example

```toml
[tool.structlog-linter]
include = ["src/**/*.py", "app/**/*.py"]
exclude = ["tests/**", "migrations/**"]

max-event-length = 40
event-case-style = "snake_case"

[tool.structlog-linter.rules]
SL006 = "error"      # promote to error
SL007 = "off"        # disable loop check
SL010 = "warning"    # enable past-tense check
```

### Top-level options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `include` | list of globs | `["**/*.py"]` | File patterns to check. |
| `exclude` | list of globs | `[]` | File patterns to skip (takes precedence over `include`). |
| `include-files` | list of paths | `[]` | Explicit file paths to check. |
| `exclude-files` | list of paths | `[]` | Explicit file paths to skip. |
| `max-event-length` | integer | `50` | Maximum event string length (SL009). |
| `event-case-style` | string | `"snake_case"` | Required event case style (SL008). |
| `loop-log-level` | string | `"debug"` | Max log level allowed inside loops (SL007). |

### Per-rule severity overrides (`[tool.structlog-linter.rules]`)

Every rule code can be set to one of:

| Value | Meaning |
|-------|---------|
| `"error"` | Report as an error (non-zero exit code). |
| `"warning"` | Report as a warning. |
| `"off"` | Disable the rule entirely. |

```toml
[tool.structlog-linter.rules]
SL001 = "error"
SL008 = "warning"
SL010 = "off"
```

### Installation

`structlog-linter` will be published as a pip-installable package:

```
pip install structlog-linter
```

The binary is written in Rust and distributed as a Python wheel (via
[PyO3](https://pyo3.rs/) / [maturin](https://www.maturin.rs/)), so no Rust
toolchain is required at install time.
