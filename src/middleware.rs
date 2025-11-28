use actix_web::{
	Error,
	dev::ServiceRequest,
	error::InternalError,
	http::StatusCode,
	web,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::{config::RelayConfig, service::RelayResponse};

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
		let response = RelayResponse::respond(
			StatusCode::UNAUTHORIZED,
			false,
			"invalid api key",
		);

		Err((InternalError::from_response("", response).into(), req))
	}
}
