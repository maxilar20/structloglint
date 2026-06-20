import structlog

log = structlog.get_logger()


def process_order(order_id, user_id):
    log.info("order_processed", order_id=order_id, user_id=user_id)


def handle_payment(order_id):
    try:
        charge(order_id)
    except TimeoutError:
        log.exception("payment_failed", order_id=order_id)


def import_products(products):
    for product in products:
        log.debug("importing", product_id=product["id"])
    log.info("import_complete", count=len(products))
