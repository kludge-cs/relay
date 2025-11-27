mod config;

use actix_web::{
	App,
	HttpServer,
	Responder,
	get,
	middleware::Logger,
	post,
	web::{self, Json},
};
use dotenvy::dotenv;
use lettre::{
	AsyncSmtpTransport,
	AsyncTransport,
	Tokio1Executor,
	message::{Mailbox, MessageBuilder, header::ContentType},
	transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};

use crate::config::{RelayConfig, RelaySMTPConfig};

#[derive(Serialize, Deserialize)]
struct RelayRequest {
	to: String,
	body: String,
	subject: String,
	name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct RelayResponse {
	success: bool,
	message: String,
}

#[derive(Clone)]
struct RelayService {
	transport: AsyncSmtpTransport<Tokio1Executor>,
	user: String,
	name: String,
}

impl RelayService {
	fn new(config: RelaySMTPConfig) -> Self {
		let transport =
			AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
				.expect("invalid SMTP server")
				.credentials(Credentials::new(
					config.user.clone(),
					config.pass.clone(),
				))
				.port(config.port)
				.build();

		Self { transport, user: config.user, name: config.name }
	}

	async fn send(
		&self,
		req: &RelayRequest,
	) -> Result<(), Box<dyn std::error::Error>> {
		let from = Mailbox::new(Some(self.name.clone()), self.user.parse()?);
		let to = Mailbox::new(req.name.clone(), req.to.parse()?);

		let message = MessageBuilder::new()
			.from(from)
			.to(to)
			.subject(&req.subject)
			.header(ContentType::TEXT_PLAIN)
			.body(req.body.clone())?;

		self.transport.send(message).await?;
		Ok(())
	}
}

#[get("/")]
pub async fn health() -> impl Responder {
	Json(RelayResponse {
		success: true,
		message: "relay is running".to_string(),
	})
}

#[post("/")]
pub async fn email(
	service: web::Data<RelayService>,
	req: Json<RelayRequest>,
) -> impl Responder {
	log::info!("sending email to={} subject={}", req.to, req.subject,);

	match service.send(&req).await {
		Ok(_) => Json(RelayResponse {
			success: true,
			message: "email sent successfully".to_string(),
		}),
		Err(e) => {
			log::error!("failed to send email {}", e);
			Json(RelayResponse {
				success: false,
				message: format!("failed to send email {}", e),
			})
		}
	}
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	dotenv().ok();
	env_logger::init();

	let config = RelayConfig::from_env();
	let addr = format!("{}:{}", config.host, config.port);

	let service = RelayService::new(config.smtp);
	log::info!("starting server at {}", addr);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(service.clone()))
			.wrap(Logger::default())
			.service(health)
			.service(email)
	})
	.bind(addr)?
	.run()
	.await
}
