use actix_web::{Error, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::config::RelayConfig;

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
		Err((ErrorUnauthorized("invalid api key"), req))
	}
}
