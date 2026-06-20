"""
SL007 — Log call inside a loop body.

Expected diagnostics: 2
    Line 17: SL007  (for loop)
    Line 22: SL007  (while loop)
"""

import structlog

log = structlog.get_logger()


def import_products(products: list):
    # OK — before the loop
    log.info("product_import_started", count=len(products))

    # SL007 — inside for loop
    for product in products:
        log.info("product_imported", product_id=product["id"])

    i = 0
    # SL007 — inside while loop
    while i < len(products):
        log.info("processing_product", index=i)
        i += 1

    # OK — after the loop
    log.info("product_import_complete", count=len(products))

    # OK — log inside for orelse (runs when loop completes normally)
    for x in []:
        pass
    else:
        log.info("empty_loop_complete")
