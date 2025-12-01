mod config;
mod error;
mod middleware;
mod service;

use actix_web::{
	App,
	HttpServer,
	Responder,
	get,
	http::StatusCode,
	middleware::Logger,
	post,
	web::{self, Json},
};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenvy::dotenv;
use validator::Validate;

use crate::{
	config::RelayConfig,
	error::RelayError,
	middleware::auth,
	service::{RelayRequest, RelayResponse, RelayService},
};

#[get("/")]
pub async fn health() -> impl Responder {
	RelayResponse::respond(StatusCode::OK, true, "relay is running")
}

#[post("/")]
pub async fn email(
	service: web::Data<RelayService>,
	req: Json<RelayRequest>,
) -> Result<impl Responder, RelayError> {
	if let Err(e) = req.validate() {
		return Err(RelayError::Validation(e.to_string()));
	}

	log::info!("sending email to={} subject={}", req.to, req.subject);

	service.send(&req).await?;

	Ok(RelayResponse::respond(StatusCode::OK, true, "email sent successfully"))
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	env_logger::init();

	let config = RelayConfig::from_env()?;
	let addr = format!("{}:{}", config.host, config.port);

	let service = RelayService::new(config.smtp.clone())?;
	log::info!("starting server at {}", addr);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(service.clone()))
			.app_data(web::Data::new(config.clone()))
			.wrap(Logger::default())
			.service(health)
			.service(
				web::scope("")
					.wrap(HttpAuthentication::bearer(auth))
					.service(email),
			)
	})
	.bind(addr)?
	.run()
	.await?;

	Ok(())
}
