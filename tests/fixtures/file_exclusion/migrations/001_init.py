import structlog

logger = structlog.get_logger()

logger.error("migration_very_long_event_message_exceeds_limit")
