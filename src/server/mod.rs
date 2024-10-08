//! REST-ful JSON API

mod context;
mod cors;
mod endpoints;
mod raw;

use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};
pub use context::Context;
pub use cors::CorsResponseOk;
use dropshot::{ApiDescription, ConfigDropshot, HttpServerStarter};
pub use raw::RawResponseOk;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use tokio::sync::RwLock;

use crate::Machine;

/// Create an API description for the server.
pub fn create_api_description() -> Result<ApiDescription<Arc<Context>>> {
    fn register_endpoints(api: &mut ApiDescription<Arc<Context>>) -> Result<(), String> {
        api.register(endpoints::ping).unwrap();
        api.register(endpoints::api_get_schema).unwrap();
        api.register(endpoints::print_file).unwrap();
        api.register(endpoints::get_machines).unwrap();
        api.register(endpoints::get_machine).unwrap();
        api.register(endpoints::get_metrics).unwrap();

        // YOUR ENDPOINTS HERE!

        Ok(())
    }

    // Describe the API.
    let tag_config = serde_json::from_str(include_str!("../../openapi/tag-config.json")).unwrap();
    let mut api = ApiDescription::new().tag_config(tag_config);

    if let Err(err) = register_endpoints(&mut api) {
        panic!("failed to register entrypoints: {}", err);
    }

    Ok(api)
}

/// Create a new Machine API Server.
pub async fn create_server(
    bind: &str,
    machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
) -> Result<(dropshot::HttpServer<Arc<Context>>, Arc<Context>)> {
    let mut api = create_api_description()?;
    let schema = get_openapi(&mut api)?;

    let config_dropshot = ConfigDropshot {
        bind_address: bind.parse()?,
        request_body_max_bytes: 107374182400, // 100 Gigiabytes.
        default_handler_task_mode: dropshot::HandlerTaskMode::CancelOnDisconnect,
        log_headers: Default::default(),
    };

    let api_context = Arc::new(Context { schema, machines });

    let server = HttpServerStarter::new(
        &config_dropshot,
        api,
        api_context.clone(),
        &slog::Logger::root(tracing_slog::TracingSlogDrain, slog::o!()),
    )
    .map_err(|error| anyhow!("failed to create server: {}", error))?
    .start();

    Ok((server, api_context))
}

/// Get the OpenAPI specification for the server.
pub fn get_openapi(api: &mut ApiDescription<Arc<Context>>) -> Result<serde_json::Value> {
    // Create the API schema.
    let mut definition = api.openapi("machine-api", clap::crate_version!());
    definition
        .description("")
        .contact_url("https://zoo.dev")
        .contact_email("machine-api@zoo.dev")
        .json()
        .map_err(|e| e.into())
}

/// Create a new Server, and serve.
pub async fn serve(bind: &str, machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>) -> Result<()> {
    let (server, _api_context) = create_server(bind, machines).await?;
    let addr: SocketAddr = bind.parse()?;

    let responder = libmdns::Responder::new().unwrap();
    let _svc = responder.register(
        "_machine-api._tcp".to_owned(),
        "Machine Api Server".to_owned(),
        addr.port(),
        &["path=/"],
    );

    // For Cloud run & ctrl+c, shutdown gracefully.
    // "The main process inside the container will receive SIGTERM, and after a grace period,
    // SIGKILL."
    // Regsitering SIGKILL here will panic at runtime, so let's avoid that.
    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    tokio::spawn(async move {
        if let Some(_sig) = signals.forever().next() {
            std::process::exit(0);
        }
    });

    server.await.map_err(|error| anyhow!("server failed: {}", error))?;

    Ok(())
}
