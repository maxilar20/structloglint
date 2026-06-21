import structlog

logger = structlog.get_logger()

logger.info("hello")
logger.warning("hi_there_12345678901234567890")
