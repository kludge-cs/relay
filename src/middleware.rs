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
	let cfg = req.app_data::<web::Data<RelayConfig>>();

	let Some(cfg) = cfg else {
		let err = RelayError::InternalServerError(
			"configuration not found.".to_string(),
		);

		return Err((
			InternalError::from_response("", err.error_response()).into(),
			req,
		));
	};
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
