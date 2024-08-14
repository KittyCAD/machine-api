use dropshot::{endpoint, HttpError, HttpResponseOk, Path, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use super::Context;
use crate::{AnyMachine, Control, MachineInfo, MachineMakeModel, MachineType, Volume};

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

/// Information regarding a connected machine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Machine {
    /// Information regarding the make and model of the attached Machine.
    pub make_model: MachineMakeModel,

    /// Information regarding the method of manufacture.
    pub machine_type: MachineType,

    /// Maximum part size that can be manufactured by this device. This may
    /// be some sort of theoretical upper bound, getting close to this limit
    /// seems like maybe a bad idea.
    ///
    /// This may be `None` if the maximum size is not knowable by the
    /// Machine API.
    ///
    /// What "close" means is up to you!
    pub max_part_volume: Option<Volume>,
}

impl Machine {
    /// Create a new API JSON Machine from a Machine struct containing the
    /// handle(s) to actually construct a part.
    pub(crate) async fn from_machine(machine: &AnyMachine) -> anyhow::Result<Self> {
        let machine_info = machine.machine_info().await?;
        Ok(Machine {
            make_model: machine_info.make_model(),
            machine_type: machine_info.machine_type(),
            max_part_volume: machine_info.max_part_volume(),
        })
    }
}

/// List available machines and their statuses
#[endpoint {
    method = GET,
    path = "/machines",
    tags = ["machines"],
}]
pub async fn get_machines(
    rqctx: RequestContext<Arc<Context>>,
) -> Result<HttpResponseOk<HashMap<String, Machine>>, HttpError> {
    tracing::info!("listing machines");
    let ctx = rqctx.context();
    let mut machines = HashMap::new();
    for (key, machine) in ctx.machines.iter() {
        let api_machine = Machine::from_machine(machine.get_machine()).await.map_err(|e| {
            tracing::warn!(
                error = format!("{:?}", e),
                "Error while fetching information for an API Machine response"
            );
            HttpError::for_internal_error(format!("{:?}", e))
        })?;
        machines.insert(key.clone(), api_machine);
    }
    Ok(HttpResponseOk(machines))
}

/// The path parameters for performing operations on an machine.
#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct MachinePathParams {
    /// The machine ID.
    pub id: String,
}

/// Get the status of a specific machine
#[endpoint {
    method = GET,
    path = "/machines/{id}",
    tags = ["machines"],
}]
pub async fn get_machine(
    rqctx: RequestContext<Arc<Context>>,
    path_params: Path<MachinePathParams>,
) -> Result<HttpResponseOk<Machine>, HttpError> {
    let params = path_params.into_inner();
    let ctx = rqctx.context();
    eprintln!("{:?}", ctx.machines.keys().collect::<Vec<_>>());

    tracing::info!(id = params.id, "finding machine");
    match ctx.machines.get(&params.id) {
        Some(machine) => Ok(HttpResponseOk(
            Machine::from_machine(machine.get_machine()).await.map_err(|e| {
                tracing::warn!(
                    error = format!("{:?}", e),
                    "Error while fetching information for an API Machine response"
                );
                HttpError::for_internal_error(format!("{:?}", e))
            })?,
        )),
        None => Err(HttpError::for_not_found(
            None,
            format!("machine not found by id: {:?}", &params.id),
        )),
    }
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
