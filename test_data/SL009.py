"""
SL009 — Event string exceeds maximum length.

Default maximum: 50 characters.

Expected diagnostics: 3
    Line 16: SL009  (84 chars)
    Line 19: SL009  (60 chars)
    Line 22: SL009  (51 chars)
"""

import structlog

log = structlog.get_logger()

# OK — short event string
log.info("user_logged_in", user_id="u_123")

# SL009 — way too long (84 chars)
log.info("the user has successfully logged into the system and was redirected to the dashboard")

# SL009 — moderately too long (60 chars)
log.warning("user_account_was_locked_after_too_many_failed_login_attempts", user_id="u_123")

# SL009 — just over the limit (51 chars)
log.info("payment_processing_completed_for_subscription_renew", order_id="o_456")

# OK — exactly at the limit (50 chars)
log.info("payment_processing_completed_for_subscription_rene", order_id="o_456")

# OK — well under the limit
log.debug("cache_hit", key="session:u_123")
