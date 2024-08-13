use dropshot::{endpoint, HttpError, HttpResponseOk, Path, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::Context;

/// Return the OpenAPI schema in JSON format.
#[endpoint {
    method = GET,
    path = "/",
    tags = ["meta"],
}]
pub async fn api_get_schema(
    rqctx: RequestContext<Arc<Context>>,
) -> Result<HttpResponseOk<serde_json::Value>, HttpError> {
    Ok(HttpResponseOk(rqctx.context().schema.clone()))
}

/// The response from the `/ping` endpoint.
#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct Pong {
    /// The pong response.
    pub message: String,
}

/** Return pong. */
#[endpoint {
    method = GET,
    path = "/ping",
    tags = ["meta"],
}]
pub async fn ping(_rqctx: RequestContext<Arc<Context>>) -> Result<HttpResponseOk<Pong>, HttpError> {
    Ok(HttpResponseOk(Pong {
        message: "pong".to_string(),
    }))
}

/** List available machines and their statuses */
#[endpoint {
    method = GET,
    path = "/machines",
    tags = ["machines"],
}]
pub async fn get_machines(
    rqctx: RequestContext<Arc<Context>>,
) -> Result<HttpResponseOk<HashMap<String, ()>>, HttpError> {
    unimplemented!();
}

/// The path parameters for performing operations on an machine.
#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct MachinePathParams {
    /// The machine ID.
    pub id: String,
}

/** Get the status of a specific machine */
#[endpoint {
    method = GET,
    path = "/machines/{id}",
    tags = ["machines"],
}]
pub async fn get_machine(
    rqctx: RequestContext<Arc<Context>>,
    path_params: Path<MachinePathParams>,
) -> Result<HttpResponseOk<()>, HttpError> {
    unimplemented!();
}

/// The response from the `/print` endpoint.
#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct PrintJobResponse {
    /// The job id used for this print.
    pub job_id: String,

    /// The parameters used for this print.
    pub parameters: PrintParameters,
}

/** Print a given file. File must be a sliceable 3D model. */
#[endpoint {
    method = POST,
    path = "/print",
    tags = ["machines"],
}]
pub(crate) async fn print_file(
    rqctx: RequestContext<Arc<Context>>,
    body_param: dropshot::MultipartBody,
) -> Result<HttpResponseOk<PrintJobResponse>, HttpError> {
    unimplemented!();
}

pub(crate) struct FileAttachment {
    file_name: Option<String>,
    content: bytes::Bytes,
}

/// Parameters for printing.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub(crate) struct PrintParameters {
    /// The machine id to print to.
    pub machine_id: String,

    /// The name for the job.
    pub job_name: String,
}

/// Possible errors returned by print endpoints.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Some error occurred when processing the multipart upload.
    #[error(transparent)]
    Multer(#[from] multer::Error),

    /// Some error occurred when (de)serializing the event.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    /// Missing attachment or event data.
    #[error("Missing file attachment or printer params.")]
    MissingFileOrParams,
}

impl From<Error> for HttpError {
    fn from(_err: Error) -> Self {
        Self::for_bad_request(None, "bad request".to_string())
    }
}

/// Parses multipart data into an request and file that we can slice and print.
#[tracing::instrument(skip_all)]
pub async fn parse_multipart_print_request(
    multipart: &mut multer::Multipart<'_>,
) -> Result<(FileAttachment, PrintParameters), Error> {
    unimplemented!();
}
