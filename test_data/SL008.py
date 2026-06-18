import structlog

log = structlog.get_logger()

# OK
log.info("profile_updated", user_id="u_123")

# SL008 - PascalCase
log.info("ProfileUpdated", user_id="u_123")

# SL008 - camelCase
log.info("profileUpdated", user_id="u_123")

# SL008 - natural language with spaces
log.info("profile updated successfully", user_id="u_123")

# SL008 - kebab-case
log.info("profile-updated", user_id="u_123")

# SL008 - SCREAMING_SNAKE
log.info("PROFILE_UPDATED", user_id="u_123")
