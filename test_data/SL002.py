"""
SL002 — F-string used as event.

Expected diagnostics: 3
    Line 13: SL002
    Line 16: SL002
    Line 19: SL002
"""
import structlog
log = structlog.get_logger()

# OK
log.info("user_logged_in", user_id="u_123", ip="1.2.3.4")

# SL002 — no interpolation, still an f-string
log.info(f"user logged in")

# SL002 — with interpolation
log.warning(f"rate limit exceeded for {'u_123'}")

# SL002 — on error
log.error(f"login failed for u_123 from 1.2.3.4", exc_info=True)

# SL002 — passing a variable
var = "u_123"
log.info(var)

# OK — f-string in a VALUE, not the event
log.info("user_logged_in", summary=f"u_123 from 1.2.3.4")

# OK — plain string
log.info("user_logged_in")
