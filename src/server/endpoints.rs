use std::{path::Path, sync::Arc};

use dropshot::{endpoint, HttpError, HttpResponseOk, RequestContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{gcode::GcodeSequence, print_manager::PrintJob, server::context::Context};

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
    tags = ["print"],
}]
pub async fn get_printers(
    rqctx: RequestContext<Arc<Context>>,
) -> Result<HttpResponseOk<Vec<crate::machine::Machine>>, HttpError> {
    let ctx = rqctx.context();
    let mut machines: Vec<crate::machine::Machine> = ctx
        .usb_printers
        .clone()
        .into_iter()
        .map(|printer| printer.into())
        .collect();
    for (_, np) in ctx.network_printers.iter() {
        machines.extend(
            np.list()
                .map_err(|_| HttpError::for_internal_error("failed to list network printers".to_owned()))?
                .into_iter()
                .map(|printer| printer.into())
                .collect::<Vec<_>>(),
        );
    }
    Ok(HttpResponseOk(machines))
}

/// The response from the `/print` endpoint.
#[derive(Deserialize, Debug, JsonSchema, Serialize)]
pub struct PrintJobResponse {
    /// The job id used for this print.
    pub job_id: String,

    /// The printer id used for this print.
    pub printer_id: String,
}

/** Print a given file. File must be a sliceable 3D model. */
#[endpoint {
    method = POST,
    path = "/print",
    tags = ["print"],
}]
pub(crate) async fn print_file(
    rqctx: RequestContext<Arc<Context>>,
    body_param: dropshot::MultipartBody,
) -> Result<HttpResponseOk<PrintJobResponse>, HttpError> {
    let mut multipart = body_param.content;
    let (file, params) = parse_multipart_print_request(&mut multipart).await?;
    let ctx = rqctx.context().clone();
    let printer_id = params.printer_id.clone();
    let printer = match ctx.usb_printers.find_by_id(printer_id.clone()) {
        Some(printer) => printer,
        None => {
            return Err(HttpError::for_bad_request(
                None,
                "printer_id must match a connected printer".to_string(),
            ))
        }
    };
    let gcode_task = tokio::task::spawn_blocking(move || {
        let dir = tempdir::TempDir::new(&printer_id)?;
        let slicer_config_path = Path::new("/home/iterion/Development/machine-api/mk3.ini");
        let stl_path = dir.path().join(file.file_name.unwrap_or("print.stl".to_string()));
        std::fs::write(&stl_path, file.content)?;
        GcodeSequence::from_stl_path(slicer_config_path, &stl_path)
    })
    .await
    .map_err(|_| HttpError::for_internal_error("failed to convert Gcode".to_owned()))?;
    let gcode = match gcode_task {
        Ok(gcode) => gcode,
        Err(err) => {
            return Err(HttpError::for_bad_request(
                None,
                format!("failed to convert file to gcode: {}", err),
            ))
        }
    };
    let job_id = uuid::Uuid::new_v4();
    let print_job = PrintJob::new(gcode, params).spawn().await;
    let mut active_jobs = ctx.active_jobs.lock().await;
    active_jobs.insert(job_id.to_string(), print_job);

    Ok(HttpResponseOk(PrintJobResponse {
        job_id: job_id.to_string(),
        printer_id: printer.id.clone(),
    }))
}

pub(crate) struct FileAttachment {
    file_name: Option<String>,
    content: bytes::Bytes,
}

/// Parameters for printing.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub(crate) struct PrintParameters {
    pub printer_id: String,
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
