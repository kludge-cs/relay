use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelayError {
	#[error("missing environment variable: {0}")]
	MissingEnvVar(String),

	#[error("internal server error: {0}")]
	InternalServerError(String),

	#[error("invalid port configuration: {0}")]
	InvalidPort(String),

	#[error("invalid SMTP server configuration")]
	InvalidSmtpServer,

	#[error("invalid email address: {0}")]
	InvalidEmail(#[from] lettre::address::AddressError),

	#[error("failed to build email message: {0}")]
	EmailBuild(#[from] lettre::error::Error),

	#[error("failed to send email: {0}")]
	EmailSend(#[from] lettre::transport::smtp::Error),

	#[error("validation error: {0}")]
	Validation(String),

	#[error("invalid API key")]
	Unauthorized,
}

impl ResponseError for RelayError {
	fn error_response(&self) -> HttpResponse {
		let (status, success, message) = match self {
			RelayError::Unauthorized => {
				(StatusCode::UNAUTHORIZED, false, self.to_string())
			}
			RelayError::Validation(_) => {
				(StatusCode::BAD_REQUEST, false, self.to_string())
			}
			_ => (StatusCode::INTERNAL_SERVER_ERROR, false, self.to_string()),
		};

		HttpResponse::build(status).json(serde_json::json!({
			"success": success,
			"message": message,
		}))
	}

	fn status_code(&self) -> StatusCode {
		match self {
			RelayError::Unauthorized => StatusCode::UNAUTHORIZED,
			RelayError::Validation(_) => StatusCode::BAD_REQUEST,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
