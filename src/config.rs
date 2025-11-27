use std::env;

pub struct RelaySMTPConfig {
	pub host: String,
	pub user: String,
	pub pass: String,
	pub name: String,
	pub port: u16,
}

pub struct RelayConfig {
	pub port: u16,
	pub key: String,
	pub host: String,
	pub smtp: RelaySMTPConfig,
}

fn env_must(var: &str) -> String {
	env::var(var).unwrap_or_else(|_| panic!("{} must be set", var))
}

fn env_or(var: &str, default: &str) -> String {
	env::var(var).unwrap_or_else(|_| default.to_string())
}

impl RelayConfig {
	pub fn from_env() -> Self {
		Self {
			host: env_or("HOST", "127.0.0.1"),
			port: env_or("PORT", "8080")
				.parse()
				.expect("PORT must be a valid number"),
			key: env_must("API_KEY"),
			smtp: RelaySMTPConfig {
				host: env_must("SMTP_HOST"),
				user: env_must("SMTP_USER"),
				pass: env_must("SMTP_PASS"),
				name: env_or("SMTP_NAME", "Relay"),
				port: env_or("SMTP_PORT", "587")
					.parse()
					.expect("SMTP_PORT must be a number"),
			},
		}
	}
}
