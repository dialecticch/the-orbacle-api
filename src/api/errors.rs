use derive_more::Display;
use hyper::StatusCode;
use rweb::{reject::Reject, warp, Rejection, Reply};

pub fn internal_error(e: impl Into<anyhow::Error>) -> Rejection {
    warp::reject::custom(ServiceError::InternalServerError(e.into()))
}

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display[fmt = "Internal Server Error: {}", _0]]
    InternalServerError(anyhow::Error),

    #[display[fmt = "BadRequest: {}", _0]]
    BadRequest(String),

    #[display[fmt = "Unauthorized"]]
    Unauthorized,

    #[display[fmt = "Unauthorized"]]
    Forbidden,
}

#[derive(Debug, rweb::Schema, serde::Serialize)]
struct ErrorJSON {
    error: String,
}

impl From<&ServiceError> for ErrorJSON {
    fn from(e: &ServiceError) -> Self {
        Self {
            error: e.to_string(),
        }
    }
}

impl From<&anyhow::Error> for ErrorJSON {
    fn from(e: &anyhow::Error) -> Self {
        Self {
            error: e.to_string(),
        }
    }
}

impl From<&str> for ErrorJSON {
    fn from(e: &str) -> Self {
        Self {
            error: e.to_string(),
        }
    }
}

impl Reject for ServiceError {}
pub async fn handle_rejection(r: Rejection) -> Result<impl Reply, Rejection> {
    if r.is_not_found() {
        return Err(warp::reject());
    }
    match r.find() {
        Some(ServiceError::BadRequest(a)) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorJSON::from(&ServiceError::BadRequest(a.to_owned()))),
            StatusCode::BAD_REQUEST,
        )),
        Some(ServiceError::Unauthorized) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorJSON::from(&ServiceError::Unauthorized)),
            StatusCode::UNAUTHORIZED,
        )),
        Some(ServiceError::Forbidden) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorJSON::from(&ServiceError::Forbidden)),
            StatusCode::FORBIDDEN,
        )),
        Some(ServiceError::InternalServerError(e)) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorJSON::from(e)),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
        None => Err(r),
    }
}
