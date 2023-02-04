use std::time::Duration;
use std::{io::Error, net::TcpListener};

use actix_web::{dev::Server, HttpServer};
use actix_web::{web, App};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::health_check;
use crate::routes::subscriptions::subscribe;
use crate::routes::subscriptions_confirm::confirm;

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(configuration: Settings) -> Result<Application, std::io::Error> {
        let pg_pool = configuration.database.get_connection_pool();
        let email_client = configuration.email_client.email_client();

        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))
        .expect(format!("Unable to bind to port {}", configuration.application.port).as_str());
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("{}:{}", configuration.application.base_url, port);
        let server = run(listener, pg_pool, email_client, base_url).await?;
        Ok(Application { port, server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub async fn run(
    listener: TcpListener,
    _pool: PgPool,
    _email_client: EmailClient,
    _base_url: String,
) -> Result<Server, Error> {
    let pool = web::Data::new(_pool);
    let email_client = web::Data::new(_email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(_base_url));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
