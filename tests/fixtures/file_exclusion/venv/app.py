import structlog

logger = structlog.get_logger()

logger.error("another_very_long_event_message_that_exceeds_limit")
