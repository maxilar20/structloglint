import structlog

log = structlog.get_logger()


def process_order(order_id, user_id):
    log.info("order_processed", order_id=order_id, user_id=user_id)

    log.info("order_processed", order_id)

    log.info(f"processing order {order_id}")

    log.info("processing order %s" % order_id)

    log.info("processing order {}".format(order_id))


def handle_payment(order_id):
    try:
        charge(order_id)
    except TimeoutError as e:
        log.error("payment_failed", order_id=order_id, error=str(e))
        log.exception("payment_failed", order_id=order_id)


def import_products(products):
    for product in products:
        log.info("product_imported", product_id=product["id"])
    log.info("import_complete", count=len(products))


def notify_user(user_id):
    log.exception("notification_failed", user_id=user_id)


def long_event():
    log.info("this_is_a_very_long_event_string_that_exceeds_the_default_max_length")


def bad_case():
    log.info("OrderProcessed", order_id="123")
    log.info("order processed successfully", order_id="123")
