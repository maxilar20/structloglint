"""
SL005 — exception() called outside an except block.

Expected diagnostics: 1
    Line 22: SL005
"""
import structlog
log = structlog.get_logger()

# OK — exception() inside except
def send_email(user_id: str):
    try:
        pass
    except Exception:
        log.exception("email_send_failed", user_id=user_id)
        raise

# OK — exception() inside nested except
def nested(user_id: str):
    try:
        try:
            pass
        except ValueError:
            log.exception("inner_failed", user_id=user_id)
    except Exception:
        log.exception("outer_failed", user_id=user_id)

# SL005 — outside any except block
def notify_user(user_id: str):
    log.exception("notification_failed", user_id=user_id)

# OK — error() outside except is fine
def archive(user_id: str):
    log.error("archive_failed", user_id=user_id)
