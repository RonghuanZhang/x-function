use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum HypervisorError {
    #[error(transparent)]
    Any(#[from] anyhow::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Invalid request: {0}")]
    InvalidRequest(String, StatusCode),
}

impl IntoResponse for HypervisorError {
    fn into_response(self) -> Response {
        let (status_code, err_msg) = match self {
            HypervisorError::Any(e) => {
                let status_code = e
                    .downcast_ref::<StatusCode>()
                    .map(ToOwned::to_owned)
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                (status_code, e.to_string())
            }
            #[rustfmt::skip]
            HypervisorError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            HypervisorError::InvalidRequest(msg, status_code) => (status_code, msg),
        };

        let err_resp = ErrorResponse { msg: err_msg };

        (status_code, axum::Json(err_resp)).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    msg: String,
}
