use std::time::Duration;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use config::Config;
use config::ConfigError;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: AppSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password.expose_secret())
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }

    pub fn get_connection_pool(self) -> sqlx::PgPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(2))
            .connect_lazy_with(self.with_db())
    }
}

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }

    pub fn email_client(self) -> EmailClient {
        let sender_email = self
            .sender()
            .expect("Invalid email id configured for sender");

        let timeout = self.timeout();
        let email_client = EmailClient::new(
            self.base_url,
            self.authorization_token,
            sender_email,
            timeout,
        );

        email_client
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Unable to resolve base path");
    let config_dir = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".to_string())
        .try_into()
        .expect("Unable to parse APP_ENVIRONMENT");

    let environment_filename = format!("{}.yaml", environment.as_str());

    let config = Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(environment_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("_"),
        )
        .build()?;

    config.try_deserialize::<Settings>()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            x => Err(format!("{} is not a supported environment", x)),
        }
    }
}
