"""
SL001 — Positional argument after event.

Expected diagnostics: 3
    Line 12: SL001
    Line 15: SL001
    Line 18: SL001
"""
import structlog
log = structlog.get_logger()

# OK
log.info("user_logged_in", user_id="u_123")
log.info("payment_complete")
log.info("payment_complete", payment_id="pay_1", duration_ms=42)

# SL001
log.info("user_logged_in", "u_123")

# SL001 — multiple positional args
log.info("payment_processed", "u_123", "ord_456", 4999)

# SL001 — mix of positional and keyword
log.warning("rate_limit_exceeded", "u_123", limit=100)

# OK — f-string in a VALUE is fine (different rule)
log.info("user_logged_in", summary=f"u_123 from 1.2.3.4")
