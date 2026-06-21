import structlog

logger = structlog.get_logger()

logger.error("very_long_cached_event_message_exceeds_limit")
