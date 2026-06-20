import logging

logger = logging.getLogger(__name__)


def process():
    logger.info("order processed for %s", "user_123")
