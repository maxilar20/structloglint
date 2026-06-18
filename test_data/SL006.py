"""
SL006 — error() inside except block without exc_info=True.

Expected diagnostics: 1
    Line 17: SL006
"""
import structlog
log = structlog.get_logger()

# SL006 — traceback silently lost
def charge_card(order_id: str):
    try:
        pass
    except TimeoutError as e:
        log.error("card_charge_timed_out", order_id=order_id, error=str(e))
        raise

# OK — explicit exc_info=True is an accepted opt-out
def refund_order(order_id: str):
    try:
        pass
    except ValueError:
        log.error("refund_validation_failed", order_id=order_id, exc_info=True)
        raise

# OK — exception() is the correct form
def cancel_order(order_id: str):
    try:
        pass
    except Exception:
        log.exception("cancel_failed", order_id=order_id)
        raise
