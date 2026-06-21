import structlog

logger = structlog.get_logger()

logger.info("hello_world")
logger.info("hi_there")
logger.info("short")

logger.warning("snake_case_event")
