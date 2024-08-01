use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context, Result};
use expectorate::assert_contents;
use pretty_assertions::assert_eq;
use test_context::{test_context, AsyncTestContext};
use testresult::TestResult;

struct ServerContext {
    config: crate::Server,
    server: dropshot::HttpServer<Arc<crate::server::context::Context>>,
    client: reqwest::Client,
}

impl ServerContext {
    pub async fn new() -> Result<Self> {
        // Find an unused port.
        let port = portpicker::pick_unused_port().ok_or_else(|| anyhow::anyhow!("no port available"))?;
        let config = crate::Server {
            address: format!("127.0.0.1:{}", port),
        };

        // Create the server in debug mode.
        let (server, _context) = crate::server::create_server(
            &config,
            &crate::Opts {
                debug: true,
                json: false,
                subcmd: crate::SubCommand::Server(config.clone()),
            },
        )
        .await?;

        // Sleep for 5 seconds while the server is comes up.
        std::thread::sleep(std::time::Duration::from_secs(5));

        Ok(ServerContext {
            config,
            server,
            client: reqwest::Client::new(),
        })
    }

    pub async fn stop(self) -> Result<()> {
        // Stop the server.
        self.server
            .close()
            .await
            .map_err(|e| anyhow::anyhow!("closing the server failed: {}", e))
    }

    pub fn get_url(&self, path: &str) -> String {
        format!("http://{}/{}", self.config.address, path.trim_start_matches('/'))
    }
}

impl AsyncTestContext for ServerContext {
    async fn setup() -> Self {
        ServerContext::new().await.unwrap()
    }

    async fn teardown(self) {
        self.stop().await.unwrap();
    }
}

#[test]
fn test_openapi() -> TestResult {
    let mut api = crate::server::create_api_description()?;
    let schema = crate::server::get_openapi(&mut api)?;
    let schema_str = serde_json::to_string_pretty(&schema)?;

    let spec: openapiv3::OpenAPI = serde_json::from_value(schema).expect("schema was not valid OpenAPI");

    assert_eq!(spec.openapi, "3.0.3");
    assert_eq!(spec.info.title, "machine-api");
    assert_eq!(spec.info.version, "0.1.0");

    // Spot check a couple of items.
    assert!(!spec.paths.paths.is_empty());
    assert!(spec.paths.paths.get("/ping").is_some());

    // Construct a string that helps us identify the organization of tags and
    // operations.
    let mut ops_by_tag = BTreeMap::<String, Vec<(String, String)>>::new();
    for (path, _, op) in spec.operations() {
        // Make sure each operation has exactly one tag. Note, we intentionally
        // do this before validating the OpenAPI output as fixing an error here
        // would necessitate refreshing the spec file again.
        assert_eq!(
            op.tags.len(),
            1,
            "operation '{}' has {} tags rather than 1",
            op.operation_id.as_ref().context("missing operation_id")?,
            op.tags.len()
        );

        ops_by_tag
            .entry(op.tags.first().context("no tags")?.to_string())
            .or_default()
            .push((
                op.operation_id.as_ref().context("missing operation_id")?.to_string(),
                path.to_string(),
            ));
    }

    let mut tags = String::new();
    for (tag, mut ops) in ops_by_tag {
        ops.sort();
        tags.push_str(&format!(r#"API operations found with tag "{tag}""#));
        tags.push_str(&format!("\n{:40} {}\n", "OPERATION ID", "URL PATH"));
        for (operation_id, path) in ops {
            tags.push_str(&format!("{operation_id:40} {path}\n"));
        }
        tags.push('\n');
    }

    // Confirm that the output hasn't changed. It's expected that we'll change
    // this file as the API evolves, but pay attention to the diffs to ensure
    // that the changes match your expectations.
    assert_contents("openapi/api.json", &schema_str);

    // When this fails, verify that operations on which you're adding,
    // renaming, or changing the tags are what you intend.
    assert_contents("openapi/api-tags.txt", &tags);

    Ok(())
}

#[test_context(ServerContext)]
#[tokio::test]
async fn test_root(ctx: &mut ServerContext) -> TestResult {
    let response = ctx.client.get(ctx.get_url("")).send().await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let text = response.text().await?;
    let expected = r#""components":{""#;
    if !text.contains(expected) {
        assert_eq!(text, expected);
    }

    Ok(())
}

#[test_context(ServerContext)]
#[tokio::test]
async fn test_ping(ctx: &mut ServerContext) -> TestResult {
    let response = ctx.client.get(ctx.get_url("ping")).send().await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await?, r#"{"message":"pong"}"#);

    Ok(())
}
