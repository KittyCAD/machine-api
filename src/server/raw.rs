use dropshot::{Body, HttpCodedResponse, HttpError};
use http::{Response, StatusCode};

/// Return an HTTP Response OK, but with CORS.
pub struct RawResponseOk(pub String);

impl HttpCodedResponse for RawResponseOk {
    type Body = String;

    const STATUS_CODE: StatusCode = StatusCode::OK;
    const DESCRIPTION: &'static str = "successful operation";
}

impl From<RawResponseOk> for Result<Response<Body>, HttpError> {
    fn from(rrok: RawResponseOk) -> Result<Response<Body>, HttpError> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "text/plain")
            .header("access-control-allow-origin", "*")
            .body(Body::from(rrok.0))?)
    }
}
