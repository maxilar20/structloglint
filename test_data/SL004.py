"""
SL004 — .format() used in event.

Expected diagnostics: 2
    Line 13: SL004
    Line 16: SL004
"""
import structlog
log = structlog.get_logger()

# OK
log.info("subscription_cancelled", user_id="u_123", reason="too_expensive")

# SL004 — positional placeholder
log.info("subscription cancelled for {}".format("u_123"))

# SL004 — named placeholders
log.warning("user {user_id} cancelled: {reason}".format(user_id="u_123", reason="too_expensive"))

# OK — .format() in a VALUE, not the event
log.info("subscription_cancelled", summary="u={} r={}".format("u_123", "expensive"))
