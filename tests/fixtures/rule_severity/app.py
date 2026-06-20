import structlog

log = structlog.get_logger()

def bad_loop():
    products = []
    for product in products:
        log.info("product_imported", product_id=product["id"])

def long_event():
    log.info("this_is_a_very_long_event_string_that_exceeds_the_default_max_length")

def wrong_case():
    log.info("OrderProcessed", order_id="123")
