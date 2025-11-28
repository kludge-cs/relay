mod config;
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
) -> impl Responder {
	if let Err(e) = req.validate() {
		return RelayResponse::respond(
			StatusCode::BAD_REQUEST,
			false,
			&format!("validation error: {}", e),
		);
	}

	log::info!("sending email to={} subject={}", req.to, req.subject);

	match service.send(&req).await {
		Ok(_) => RelayResponse::respond(
			StatusCode::OK,
			true,
			"email sent successfully",
		),
		Err(e) => {
			log::error!("failed to send email {}", e);

			RelayResponse::respond(
				StatusCode::INTERNAL_SERVER_ERROR,
				false,
				&format!("failed to send email {}", e),
			)
		}
	}
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();
	env_logger::init();

	let config = RelayConfig::from_env();
	let addr = format!("{}:{}", config.host, config.port);

	let service = RelayService::new(config.smtp.clone());
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
	.await
}
