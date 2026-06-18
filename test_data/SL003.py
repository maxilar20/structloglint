"""
SL003 — %-formatting used in event.

Expected diagnostics: 2
    Line 13: SL003
    Line 16: SL003
"""
import structlog
log = structlog.get_logger()

# OK
log.info("user_signed_up", username="alice", plan="pro")

# SL003 — single substitution
log.info("user %s signed up" % "alice")

# SL003 — tuple form
log.info("user %s on plan %s" % ("alice", "pro"))

# OK — % in a VALUE, not the event
log.info("user_signed_up", detail="%s/%s" % ("alice", "pro"))
