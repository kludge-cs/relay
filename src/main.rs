use std::env;

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
	fn new(
		server: String,
		user: String,
		pass: String,
		name: String,
		port: u16,
	) -> Self {
		let transport = if port == 25 {
			AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&server)
				.port(port)
				.build()
		} else {
			AsyncSmtpTransport::<Tokio1Executor>::relay(&server)
				.expect("invalid SMTP server")
				.credentials(Credentials::new(user.clone(), pass))
				.port(port)
				.build()
		};

		RelayService { transport, user, name }
	}

	async fn send(
		&self,
		req: &RelayRequest,
	) -> Result<(), Box<dyn std::error::Error>> {
		let from = Mailbox::new(Some(self.name.clone()), self.user.parse()?);
		let to = Mailbox::new(req.name.clone(), req.to.parse()?);

		let email = MessageBuilder::new()
			.from(from)
			.to(to)
			.subject(&req.subject)
			.header(ContentType::TEXT_PLAIN)
			.body(req.body.clone())?;

		self.transport.send(email).await?;
		Ok(())
	}
}

#[get("/")]
pub async fn health_check() -> impl Responder {
	Json(RelayResponse {
		success: true,
		message: "relay is running".to_string(),
	})
}

#[post("/")]
pub async fn send_email(
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

	let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
	let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
	let addr = format!("{}:{}", host, port);

	let email_service = RelayService::new(
		env::var("SMTP_SERVER").expect("SMTP_SERVER must be set"),
		env::var("SMTP_USER").expect("SMTP_USER must be set"),
		env::var("SMTP_PASS").expect("SMTP_PASS must be set"),
		env::var("SMTP_NAME").unwrap_or_else(|_| "Relay".to_string()),
		env::var("SMTP_PORT")
			.unwrap_or_else(|_| "587".to_string())
			.parse()
			.expect("SMTP_PORT must be a number"),
	);

	log::info!("starting server at {}", addr);

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(email_service.clone()))
			.wrap(Logger::default())
			.service(health_check)
			.service(send_email)
	})
	.bind(addr)?
	.run()
	.await
}
