use std::env;

use crate::error::RelayError;

#[derive(Clone)]
pub struct RelaySMTPConfig {
	pub host: String,
	pub user: String,
	pub pass: String,
	pub name: String,
	pub port: u16,
}

#[derive(Clone)]
pub struct RelayConfig {
	pub port: u16,
	pub key: String,
	pub host: String,
	pub smtp: RelaySMTPConfig,
}

fn env_must(var: &str) -> Result<String, RelayError> {
	env::var(var).map_err(|_| RelayError::MissingEnvVar(var.to_string()))
}

fn env_or(var: &str, default: &str) -> String {
	env::var(var).unwrap_or_else(|_| default.to_string())
}

impl RelayConfig {
	pub fn from_env() -> Result<Self, RelayError> {
		Ok(Self {
			host: env_or("HOST", "127.0.0.1"),
			port: env_or("PORT", "8080").parse().map_err(|_| {
				RelayError::InvalidPort(
					"PORT must be a valid number".to_string(),
				)
			})?,
			key: env_must("API_KEY")?,
			smtp: RelaySMTPConfig {
				host: env_must("SMTP_HOST")?,
				user: env_must("SMTP_USER")?,
				pass: env_must("SMTP_PASS")?,
				name: env_or("SMTP_NAME", "Relay"),
				port: env_or("SMTP_PORT", "587").parse().map_err(|_| {
					RelayError::InvalidPort(
						"SMTP_PORT must be a number".to_string(),
					)
				})?,
			},
		})
	}
}
