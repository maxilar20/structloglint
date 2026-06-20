"""
SL010 — Event string should use past tense.

This rule is off by default and must be explicitly enabled.

Expected diagnostics: 6
    Line 16: SL010  (present progressive: "logging_in")
    Line 19: SL010  (present progressive: "processing")
    Line 22: SL010  (imperative: "send")
    Line 25: SL010  (imperative: "create")
    Line 28: SL010  (present progressive: "retrying")
    Line 31: SL010  (imperative: "delete")
"""

import structlog

log = structlog.get_logger()

# SL010 — present progressive (-ing)
log.info("user_logging_in", user_id="u_123")

# SL010 — present progressive (-ing)
log.info("processing_payment", order_id="o_456")

# SL010 — imperative / base form
log.warning("send_email", user_id="u_123")

# SL010 — imperative / base form
log.info("create_user", username="alice")

# SL010 — present progressive (-ing)
log.error("retrying_connection", attempt=3)

# SL010 — imperative / base form
log.info("delete_expired_sessions", count=42)

# OK — past tense
log.info("user_logged_in", user_id="u_123")

# OK — past tense
log.info("payment_processed", order_id="o_456")

# OK — past tense
log.info("email_sent", user_id="u_123")

# OK — past tense
log.info("order_cancelled", order_id="o_789")

# OK — past tense
log.debug("cache_invalidated", key="session:u_123")

# OK — past tense (irregular verb)
log.info("connection_lost", host="db.internal")

# OK — past participle used as adjective-like event
log.warning("rate_limit_exceeded", user_id="u_123", limit=100)

# OK — past tense
log.info("user_created", username="alice")
