use std::sync::Arc;

use dropshot::{endpoint, HttpError, Path, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Context, CorsResponseOk, RawResponseOk};
use crate::{
    AnyMachine, Control, DesignFile, MachineInfo, MachineMakeModel, MachineState, MachineType, TemporaryFile, Volume,
};

/// Return the OpenAPI schema in JSON format.
#[endpoint {
    method = GET,
    path = "/",
    tags = ["meta"],
}]
pub async fn api_get_schema(
    rqctx: RequestContext<Arc<Context>>,
) -> Result<CorsResponseOk<serde_json::Value>, HttpError> {
    Ok(CorsResponseOk(rqctx.context().schema.clone()))
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
pub async fn ping(_rqctx: RequestContext<Arc<Context>>) -> Result<CorsResponseOk<Pong>, HttpError> {
    Ok(CorsResponseOk(Pong {
        message: "pong".to_string(),
    }))
}

/// Extra machine-specific information regarding a connected machine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ExtraMachineInfoResponse {
    Moonraker {},
    Usb {},
    Bambu {
        /// The current stage of the machine as defined by Bambu which can include errors, etc.
        current_stage: Option<bambulabs::message::Stage>,
        /// The nozzle diameter of the machine.
        nozzle_diameter: bambulabs::message::NozzleDiameter,
        // Only run in debug mode. This is just to help us know what information we have.
        #[cfg(debug_assertions)]
        #[cfg(not(test))]
        /// The raw status message from the machine.
        raw_status: bambulabs::message::PushStatus,
    },
}

/// Information regarding a connected machine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MachineInfoResponse {
    /// Machine Identifier (ID) for the specific Machine.
    pub id: String,

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

    /// Status of the printer -- be it printing, idle, or unreachable. This
    /// may dictate if a machine is capable of taking a new job.
    pub state: MachineState,

    /// Additional, per-machine information which is specific to the
    /// underlying machine type.
    pub extra: Option<ExtraMachineInfoResponse>,
}

impl MachineInfoResponse {
    /// Create a new API JSON Machine from a Machine struct containing the
    /// handle(s) to actually construct a part.
    pub(crate) async fn from_machine(id: &str, machine: &AnyMachine) -> anyhow::Result<Self> {
        let machine_info = machine.machine_info().await?;
        Ok(MachineInfoResponse {
            id: id.to_owned(),
            make_model: machine_info.make_model(),
            machine_type: machine_info.machine_type(),
            max_part_volume: machine_info.max_part_volume(),
            state: machine.state().await?,
            extra: match machine {
                AnyMachine::Moonraker(_) => Some(ExtraMachineInfoResponse::Moonraker {}),
                AnyMachine::Usb(_) => Some(ExtraMachineInfoResponse::Usb {}),
                AnyMachine::BambuX1Carbon(bambu) => {
                    let status = bambu
                        .get_status()?
                        .ok_or_else(|| anyhow::anyhow!("no status for bambu"))?;
                    Some(ExtraMachineInfoResponse::Bambu {
                        current_stage: status.stg_cur,
                        nozzle_diameter: status.nozzle_diameter,
                        #[cfg(debug_assertions)]
                        #[cfg(not(test))]
                        raw_status: status,
                    })
                }
                _ => None,
            },
        })
    }

    /// Return an API JSON Machine from a Machine struct, returning a 500
    /// if the machine fails to enumerate.
    pub(crate) async fn from_machine_http(id: &str, machine: &AnyMachine) -> Result<MachineInfoResponse, HttpError> {
        Self::from_machine(id, machine).await.map_err(|e| {
            tracing::warn!(
                error = format!("{:?}", e),
                "Error while fetching information for an API Machine response"
            );
            HttpError::for_internal_error(format!("{:?}", e))
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
) -> Result<CorsResponseOk<Vec<MachineInfoResponse>>, HttpError> {
    tracing::info!("listing machines");
    let ctx = rqctx.context();
    let mut machines = vec![];
    for (key, machine) in ctx.machines.read().await.iter() {
        let api_machine = MachineInfoResponse::from_machine_http(key, machine.read().await.get_machine()).await?;
        machines.push(api_machine);
    }
    Ok(CorsResponseOk(machines))
}

/// List available machines and their statuses
#[endpoint {
    method = GET,
    path = "/metrics",
    tags = ["hidden"],
}]
pub async fn get_metrics(rqctx: RequestContext<Arc<Context>>) -> Result<RawResponseOk, HttpError> {
    let ctx = rqctx.context();
    let mut response = String::new();

    prometheus_client::encoding::text::encode(&mut response, &ctx.registry)
        .map_err(|e| HttpError::for_internal_error(format!("{:?}", e)))?;

    Ok(RawResponseOk(response))
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
) -> Result<CorsResponseOk<MachineInfoResponse>, HttpError> {
    let params = path_params.into_inner();
    let ctx = rqctx.context();

    tracing::info!(id = params.id, "finding machine");
    match ctx.machines.read().await.get(&params.id) {
        Some(machine) => Ok(CorsResponseOk(
            MachineInfoResponse::from_machine_http(&params.id, machine.read().await.get_machine()).await?,
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
) -> Result<CorsResponseOk<PrintJobResponse>, HttpError> {
    let mut multipart = body_param.content;
    let (file, params) = parse_multipart_print_request(&mut multipart).await?;
    let ctx = rqctx.context().clone();
    let machine_id = params.machine_id.clone();
    let job_id = uuid::Uuid::new_v4();
    let job_name = &params.job_name;

    let machines = ctx.machines.read().await;
    let machine = match machines.get(&machine_id) {
        Some(machine) => machine,
        None => {
            tracing::warn!(id = machine_id, "machine not found");
            return Err(HttpError::for_not_found(
                None,
                format!("machine not found by id: {:?}", machine_id),
            ));
        }
    };

    let filepath = std::env::temp_dir().join(format!(
        "{}_{}",
        job_id.simple(),
        file.file_name.unwrap_or("file".to_string())
    ));
    tracing::info!(path = format!("{:?}", filepath), "Writing file to disk");

    // TODO: we likely want to use the kittycad api to convert the file to the right format if its
    // not already an stl file.

    tokio::fs::write(&filepath, file.content).await.map_err(|e| {
        tracing::error!(error = format!("{:?}", e), "failed to write stl file");
        HttpError::for_bad_request(None, "failed to write stl file".to_string())
    })?;

    let tmpfile = TemporaryFile::new(&filepath)
        .await
        .map_err(|e| HttpError::for_internal_error(format!("{:?}", e)))?;

    machine
        .write()
        .await
        .build(job_name, &DesignFile::Stl(tmpfile.path().to_path_buf()))
        .await
        .map_err(|e| {
            tracing::warn!(error = format!("{:?}", e), "failed to build file");
            // Get the last 100 characters of the error message
            let mut error_message = format!("{:?}", e);
            if error_message.len() > 100 {
                error_message = error_message
                    .chars()
                    .rev()
                    .take(100)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();
            }
            HttpError::for_bad_request(
                None,
                format!(
                    "Your print failed, it might be too big for the slicer or something else. {}",
                    error_message
                ),
            )
        })?;

    Ok(CorsResponseOk(PrintJobResponse {
        job_id: job_id.to_string(),
        parameters: params,
    }))
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
    let mut maybe_file = None;
    let mut maybe_params = None;

    while let Some(field) = multipart.next_field().await? {
        if let Some(name) = field.name() {
            if name == "file" {
                maybe_file = Some(FileAttachment {
                    file_name: field.file_name().map(str::to_string),
                    content: field.bytes().await?,
                })
            } else if name == "params" {
                let params = field.json::<PrintParameters>().await?;
                maybe_params = Some(params);
            }
        } else {
            // ignore if the field has no name
            continue;
        }
    }

    if let (Some(file), Some(params)) = (maybe_file, maybe_params) {
        Ok((file, params))
    } else {
        return Err(Error::MissingFileOrParams);
    }
}
