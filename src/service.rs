use actix_web::{HttpResponse, http::StatusCode};
use lettre::{
	AsyncSmtpTransport,
	AsyncTransport,
	Tokio1Executor,
	message::{Mailbox, MessageBuilder, header::ContentType},
	transport::smtp::authentication::Credentials,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{config::RelaySMTPConfig, error::RelayError};

#[derive(Serialize, Deserialize, Validate)]
pub struct RelayRequest {
	#[validate(email)]
	pub to: String,

	#[validate(length(min = 1, max = 256, message = "cannot be empty"))]
	pub subject: String,

	#[validate(length(min = 1, max = 256_000, message = "cannot be empty"))]
	pub body: String,

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
	pub fn respond(
		status: StatusCode,
		success: bool,
		message: &str,
	) -> HttpResponse {
		HttpResponse::build(status)
			.json(Self { success, message: message.to_string() })
	}
}

impl RelayService {
	pub fn new(config: RelaySMTPConfig) -> Result<Self, RelayError> {
		let transport =
			AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
				.map_err(|_| RelayError::InvalidSmtpServer)?
				.credentials(Credentials::new(
					config.user.clone(),
					config.pass.clone(),
				))
				.port(config.port)
				.build();

		Ok(Self { transport, user: config.user, name: config.name })
	}

	pub async fn send(&self, req: &RelayRequest) -> Result<(), RelayError> {
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
