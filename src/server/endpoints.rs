use std::{collections::HashMap, sync::Arc};

use dropshot::{endpoint, HttpError, HttpResponseOk, Path, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{print_manager::PrintJob, server::context::Context};

/**
 * Return the OpenAPI schema in JSON format.
 */
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
) -> Result<HttpResponseOk<HashMap<String, crate::machine::Machine>>, HttpError> {
    let ctx = rqctx.context();
    let machines = ctx.list_machines().map_err(|e| {
        tracing::error!("failed to list machines: {:?}", e);
        HttpError::for_bad_request(None, "failed to list machines".to_string())
    })?;
    Ok(HttpResponseOk(machines))
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
) -> Result<HttpResponseOk<crate::machine::Message>, HttpError> {
    let params = path_params.into_inner();
    let ctx = rqctx.context();
    let machine = ctx
        .find_machine_handle_by_id(&params.id)
        .map_err(|e| {
            tracing::error!("failed to find machine by id: {:?}", e);
            HttpError::for_bad_request(None, format!("machine not found by id: {:?}", params.id))
        })?
        .ok_or_else(|| {
            tracing::error!("machine not found by id: {:?}", params.id);
            HttpError::for_not_found(None, format!("machine not found by id: {:?}", params.id))
        })?;

    let message = machine.status().await.map_err(|e| {
        tracing::error!("failed to get machine status: {:?}", e);
        HttpError::for_bad_request(None, "failed to get machine status".to_string())
    })?;

    Ok(HttpResponseOk(message))
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
    let mut multipart = body_param.content;
    let (file, params) = parse_multipart_print_request(&mut multipart).await?;
    let ctx = rqctx.context().clone();
    let machine_id = params.machine_id.clone();
    let job_id = uuid::Uuid::new_v4();

    let machine = ctx
        .find_machine_handle_by_id(&machine_id)
        .map_err(|e| {
            tracing::error!("failed to find machine by id: {:?}", e);
            HttpError::for_bad_request(None, format!("machine not found by id: {:?}", machine_id))
        })?
        .ok_or_else(|| {
            tracing::error!("machine not found by id: {:?}", machine_id);
            HttpError::for_not_found(None, format!("machine not found by id: {:?}", machine_id))
        })?;
    let filepath = std::env::temp_dir().join(format!("{}-{}", job_id, file.file_name.unwrap_or("file".to_string())));
    // TODO: we likely want to use the kittycad api to convert the file to the right format if its
    // not already an stl file.
    tokio::fs::write(&filepath, file.content).await.map_err(|e| {
        tracing::error!("failed to write stl file: {:?}", e);
        HttpError::for_bad_request(None, "failed to write stl file".to_string())
    })?;

    match machine {
        crate::machine::MachineHandle::UsbPrinter(_) => {
            let print_job = PrintJob {
                file: filepath,
                machine,
                job_name: params.job_name.to_string(),
            };
            let handle = print_job.spawn().await;
            let mut active_jobs = ctx.active_jobs.lock().await;
            active_jobs.insert(job_id.to_string(), handle);
        }
        crate::machine::MachineHandle::NetworkPrinter(printer) => {
            let result = printer
                .client
                .slice_and_print(&params.job_name, &filepath)
                .await
                .map_err(|e| {
                    tracing::error!("failed to print file: {:?}", e);
                    HttpError::for_bad_request(None, "Failed to print file. This could be because the print is too big for the build plate or too high.".to_string())

                })?;

            tracing::info!("result: {:?}", result);
        }
    }

    Ok(HttpResponseOk(PrintJobResponse {
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
