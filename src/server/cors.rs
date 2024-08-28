use dropshot::{HttpCodedResponse, HttpError};
use http::{Response, StatusCode};
use hyper::Body;
use schemars::JsonSchema;
use serde::Serialize;

/// Return an HTTP Response OK, but with CORS.
pub struct CorsResponseOk<T>(pub T);

impl<InnerT> HttpCodedResponse for CorsResponseOk<InnerT>
where
    InnerT: Serialize,
    InnerT: JsonSchema,
    InnerT: Send,
    InnerT: Sync,
    InnerT: 'static,
{
    type Body = InnerT;

    const STATUS_CODE: StatusCode = StatusCode::OK;
    const DESCRIPTION: &'static str = "successful operation";
}

impl<InnerT> From<CorsResponseOk<InnerT>> for Result<Response<Body>, HttpError>
where
    InnerT: Serialize,
    InnerT: JsonSchema,
{
    fn from(crok: CorsResponseOk<InnerT>) -> Result<Response<Body>, HttpError> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header("access-control-allow-origin", "*")
            .body(
                serde_json::to_vec(&crok.0)
                    .map_err(|e| {
                        tracing::warn!(error = format!("{:?}", e), "failed to construct response");
                        HttpError::for_internal_error(format!("{:?}", e))
                    })?
                    .into(),
            )?)
    }
}
