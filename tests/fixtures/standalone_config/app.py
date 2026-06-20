import structlog

log = structlog.get_logger()


def good_kebab():
    log.info("order-processed", order_id="123")


def bad_snake():
    log.info("order_processed", order_id="123")


def event_over_40():
    log.info("this-event-is-definitely-over-forty-characters-long", order_id="123")


def debug_in_loop_ok():
    items = []
    for item in items:
        log.debug("processing", item_id=item)


def info_in_loop_ok_with_info_min():
    items = []
    for item in items:
        log.info("processing", item_id=item)


def warning_in_loop_fails():
    items = []
    for item in items:
        log.warning("slow-processing", item_id=item)
