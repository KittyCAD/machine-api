use anyhow::Result;

/**
 * Application-specific context (state shared by handler functions)
 */
pub struct Context {
    pub schema: serde_json::Value,
    pub logger: slog::Logger,
    pub settings: crate::Server,
}

impl Context {
    /**
     * Return a new Context.
     */
    pub async fn new(schema: serde_json::Value, logger: slog::Logger, settings: crate::Server) -> Result<Context> {
        // Create the context.
        Ok(Context {
            schema,
            logger,
            settings,
        })
    }
}
