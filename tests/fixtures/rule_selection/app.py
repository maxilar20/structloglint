import structlog

log = structlog.get_logger()

def bad():
    log.info("order_processed", "extra")
    log.info(f"processing order 42")
    log.info("processing order %s" % 42)
