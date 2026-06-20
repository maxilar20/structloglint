"""
example_structlog.py

A realistic e-commerce order processing module.
Used as a test fixture for structlog-lint.

Each violation is tagged with the rule it should trigger:
    SL001 - positional argument after event (not keyword)
    SL002 - f-string used as event
    SL003 - %-formatting used in event
    SL004 - .format() used in event
    SL005 - exception() called outside an except block
    SL006 - error() called inside except block instead of exception()
    SL007 - debug/info/warning inside a tight loop body
    SL008 - event name is not snake_case
    SL009 - event name exceeds maximum length

Lines marked  # OK  are correct and should produce no diagnostics.
"""

import sys
import time

import structlog
import structlog as sl  # alias — name heuristic handles the logger variable name
from structlog import get_logger

log = structlog.get_logger()  # OK - standard module-level logger
logger = get_logger()  # OK - from-import form
other_logger = sl.get_logger()  # OK - aliased import form

# ---------------------------------------------------------------------------
# 1. IMPORT / ALIAS SMOKE TEST
#    All three logger names must be recognised by the name heuristic.
# ---------------------------------------------------------------------------

log.debug("startup", version="1.0.0")  # OK
logger.debug("startup", version="1.0.0")  # OK
other_logger.debug("startup", version="1.0.0")  # OK


# This is NOT a structlog logger — should never trigger any SL rules.
class FakeLogger:
    def info(self, event, *args, **kwargs):
        print(event)


fake = FakeLogger()
fake.info("something happened", "positional")  # OK — not a structlog logger


# ---------------------------------------------------------------------------
# 2. SL001 — POSITIONAL ARGUMENT AFTER EVENT
# ---------------------------------------------------------------------------


def process_payment(user_id: str, order_id: str, amount_cents: int):
    # OK — keyword arguments only
    log.info(
        "payment_processing_started",
        user_id=user_id,
        order_id=order_id,
        amount_cents=amount_cents,
    )

    # SL001 — user_id passed positionally
    log.info("payment_processing_started", user_id)

    # SL001 — multiple positional args
    log.info("payment_processing_started", user_id, order_id, amount_cents)

    # SL001 — mix of positional and keyword
    log.warning("large_order_flagged", amount_cents, threshold=50_000)  # SL001

    # OK — event is the only positional arg (that's fine)
    log.info("payment_complete")

    # OK — keyword args after event
    log.info("payment_complete", payment_id="ch_123", duration_ms=42)


# ---------------------------------------------------------------------------
# 3. SL002 — F-STRING AS EVENT
# ---------------------------------------------------------------------------


def handle_login(user_id: str, ip: str):
    # OK
    log.info("user_logged_in", user_id=user_id, ip=ip)

    # SL002 — f-string event, context baked in, can't filter by event name
    log.info(f"user {user_id} logged in")

    # SL002 — even a trivial f-string with no interpolation is flagged
    log.warning(f"rate limit exceeded")

    # SL002 — nested call, still an f-string
    log.error(f"login failed for {user_id} from {ip}", exc_info=True)

    # OK — f-string in a VALUE is fine
    log.info("login_attempt", summary=f"{user_id} from {ip}")


# ---------------------------------------------------------------------------
# 4. SL003 — %-FORMATTING IN EVENT
# ---------------------------------------------------------------------------


def handle_signup(username: str, plan: str):
    # OK
    log.info("user_signed_up", username=username, plan=plan)

    # SL003 — old-style % formatting in event string
    log.info("user %s signed up" % username)

    # SL003 — multiple substitutions
    log.info("user %s signed up on plan %s" % (username, plan))

    # OK — % in a value, not the event
    log.info("user_signed_up", detail="%s/%s" % (username, plan))


# ---------------------------------------------------------------------------
# 5. SL004 — .format() IN EVENT
# ---------------------------------------------------------------------------


def cancel_subscription(user_id: str, reason: str):
    # OK
    log.info("subscription_cancelled", user_id=user_id, reason=reason)

    # SL004 — .format() in event
    log.info("subscription cancelled for {}".format(user_id))

    # SL004 — named .format()
    log.warning(
        "user {user_id} cancelled: {reason}".format(user_id=user_id, reason=reason)
    )

    # OK — .format() in a value is fine
    log.info(
        "subscription_cancelled", summary="user={} reason={}".format(user_id, reason)
    )


# ---------------------------------------------------------------------------
# 6. SL005 — exception() CALLED OUTSIDE EXCEPT BLOCK
# ---------------------------------------------------------------------------


def send_welcome_email(user_id: str):
    # OK — exception() inside except
    try:
        _send_email(user_id)
    except Exception:
        log.exception("email_send_failed", user_id=user_id)
        raise

    # SL005 — exception() called outside except (no active exception to capture)
    log.exception("email_send_failed", user_id=user_id)

    # OK — error() outside except is fine
    log.error("email_send_failed", user_id=user_id)


# ---------------------------------------------------------------------------
# 7. SL006 — error() INSIDE EXCEPT INSTEAD OF exception()
# ---------------------------------------------------------------------------


def charge_card(order_id: str, amount_cents: int):
    try:
        result = _stripe_charge(order_id, amount_cents)
        log.info("card_charged", order_id=order_id, charge_id=result["id"])  # OK
        return result
    except TimeoutError as e:
        # SL006 — traceback silently lost; use log.exception() instead
        log.error("card_charge_timed_out", order_id=order_id, error=str(e))
        raise
    except Exception as e:
        # SL006 — same problem
        log.error("card_charge_failed", order_id=order_id)
        raise
    finally:
        # OK — info/warning in finally is fine
        log.debug("card_charge_attempt_complete", order_id=order_id)


def refund_order(order_id: str):
    try:
        _issue_refund(order_id)
    except ValueError as e:
        # OK — explicitly passing exc_info bypasses the rule
        log.error("refund_validation_failed", order_id=order_id, exc_info=True)
        raise
    except Exception:
        # OK — this is the correct form
        log.exception("refund_failed", order_id=order_id)
        raise


# ---------------------------------------------------------------------------
# 8. SL007 — LOGGING INSIDE A TIGHT LOOP
# ---------------------------------------------------------------------------


def import_products(products: list):
    # SL007 — log inside a for loop body
    for product in products:
        log.info("product_imported", product_id=product["id"])  # SL007

    # SL007 — log inside while loop
    i = 0
    while i < len(products):
        log.debug("processing_product", index=i)  # SL007
        i += 1

    # OK — log before/after the loop
    log.info("product_import_started", count=len(products))
    results = [_import_one(p) for p in products]
    log.info("product_import_complete", count=len(products), failed=results.count(None))


def retry_failed_jobs(jobs: list):
    failures = []
    for job in jobs:
        try:
            _run_job(job)
        except Exception:
            # SL007 — inside loop AND inside except; SL006 would also fire
            log.error("job_failed", job_id=job["id"])  # SL006 + SL007
            failures.append(job)

    if failures:
        # OK — summary log after the loop
        log.warning(
            "jobs_failed", count=len(failures), job_ids=[j["id"] for j in failures]
        )


# ---------------------------------------------------------------------------
# 9. SL008 — EVENT NAME NOT SNAKE_CASE
# ---------------------------------------------------------------------------


def update_profile(user_id: str):
    # OK
    log.info("profile_updated", user_id=user_id)

    # SL008 — PascalCase
    log.info("ProfileUpdated", user_id=user_id)

    # SL008 — camelCase
    log.info("profileUpdated", user_id=user_id)

    # SL008 — spaces (natural language)
    log.info("profile updated successfully", user_id=user_id)

    # SL008 — kebab-case
    log.info("profile-updated", user_id=user_id)

    # SL008 — SCREAMING_SNAKE is arguably fine but let's flag it
    log.info("PROFILE_UPDATED", user_id=user_id)


# ---------------------------------------------------------------------------
# 10. BOUND LOGGER — use conventional names (log/logger) for detection
# ---------------------------------------------------------------------------


def handle_order(user_id: str, order_id: str):
    # Bind context once, use throughout — name must match heuristic
    logger = log.bind(user_id=user_id, order_id=order_id)  # OK

    logger.info("order_received")  # OK
    logger.debug("order_validated", item_count=3)  # OK

    # SL001 — bound logger, positional arg
    logger.warning("order_flagged", "suspicious_pattern")  # SL001

    # SL002 — bound logger, f-string event
    logger.error(f"order {order_id} failed")  # SL002

    # Chained bind — rebind to same name (best practice)
    logger = logger.bind(step="payment")
    logger.info("payment_step_started")  # OK
    logger.info("payment step started")  # SL008


# ---------------------------------------------------------------------------
# 11. REAL-WORLD REALISTIC FUNCTION — mix of good and bad
# ---------------------------------------------------------------------------


def fulfill_order(order_id: str, warehouse_id: str) -> bool:
    """
    Attempts to fulfil an order from a given warehouse.
    Contains a deliberate mix of correct and incorrect log calls.
    """
    logger = log.bind(order_id=order_id, warehouse_id=warehouse_id)
    start = time.monotonic()

    logger.info("fulfillment_started")  # OK

    try:
        inventory = _check_inventory(order_id, warehouse_id)
    except ConnectionError:
        logger.exception("inventory_check_failed")  # OK
        return False

    if not inventory["available"]:
        # OK
        logger.warning(
            "inventory_unavailable",
            requested=inventory["requested"],
            available=inventory["available"],
        )
        return False

    items = inventory["items"]
    for item in items:
        # SL007 — inside loop
        logger.debug("picking_item", sku=item["sku"], qty=item["qty"])

    try:
        shipment = _create_shipment(order_id, items)
    except ValueError as e:
        # SL006 — should be exception()
        logger.error("shipment_creation_failed", error=str(e))
        return False
    except Exception:
        logger.exception("shipment_creation_error")  # OK
        return False

    elapsed_ms = (time.monotonic() - start) * 1000

    # SL001 — shipment_id passed positionally
    logger.info("fulfillment_complete", shipment["id"], duration_ms=elapsed_ms)

    return True


# ---------------------------------------------------------------------------
# HELPERS (stubs so the file is self-contained)
# ---------------------------------------------------------------------------


def _send_email(user_id):
    pass


def _stripe_charge(order_id, amount_cents):
    return {"id": "ch_123"}


def _issue_refund(order_id):
    pass


def _import_one(product):
    return product


def _run_job(job):
    pass


def _check_inventory(order_id, warehouse_id):
    return {"available": True, "requested": 1, "available": 1, "items": []}


def _create_shipment(order_id, items):
    return {"id": "shp_123"}
