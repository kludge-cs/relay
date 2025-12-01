use actix_web::{
	Error,
	ResponseError,
	dev::ServiceRequest,
	error::InternalError,
	web,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::{config::RelayConfig, error::RelayError};

pub async fn auth(
	req: ServiceRequest,
	cred: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
	let cfg = req
		.app_data::<web::Data<RelayConfig>>()
		.expect("RelayConfig not found");

	if cred.token() == cfg.key {
		Ok(req)
	} else {
		Err((
			InternalError::from_response(
				"",
				RelayError::Unauthorized.error_response(),
			)
			.into(),
			req,
		))
	}
}
