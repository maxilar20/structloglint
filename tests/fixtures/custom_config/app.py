import structlog

log = structlog.get_logger()


def good_camel_case():
    log.info("orderProcessed", order_id="123")
    log.info("userLoggedIn", user_id="456")


def bad_snake_case():
    log.info("order_processed", order_id="123")


def event_under_50_but_over_30():
    log.info("thisEventIsUnderFiftyCharacters", order_id="123")


def event_over_50():
    log.info("thisEventStringIsDefinitelyWayOverFiftyCharactersInLength", order_id="123")


def info_in_loop_is_ok_with_warning_level():
    products = []
    for product in products:
        log.info("productImported", product_id=product["id"])


def warning_in_loop_fails():
    products = []
    for product in products:
        log.warning("productImportSlow", product_id=product["id"])
