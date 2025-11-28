use actix_web::web::Json;
use lettre::{
	AsyncSmtpTransport,
	AsyncTransport,
	Tokio1Executor,
	message::{Mailbox, MessageBuilder, header::ContentType},
	transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};

use crate::config::RelaySMTPConfig;

#[derive(Serialize, Deserialize)]
pub struct RelayRequest {
	pub to: String,
	pub body: String,
	pub subject: String,
	pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RelayResponse {
	pub success: bool,
	pub message: String,
}

#[derive(Clone)]
pub struct RelayService {
	pub transport: AsyncSmtpTransport<Tokio1Executor>,
	pub user: String,
	pub name: String,
}

impl RelayResponse {
	pub fn json(success: bool, message: &str) -> Json<RelayResponse> {
		Json(Self { success, message: message.to_string() })
	}
}

impl RelayService {
	pub fn new(config: RelaySMTPConfig) -> Self {
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

	pub async fn send(
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
