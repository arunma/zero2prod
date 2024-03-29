use std::net::TcpListener;

use actix_web::HttpResponse;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::{run, Application},
    telemetry::{get_subscriber, init_subscriber},
};
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub addr: String,
    pub port: u16,
    pub pool: PgPool,
    pub email_server: MockServer,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        let address = format!("{}/subscribe", &self.addr);
        println!("Address in post_subscription is : {}", &address);
        reqwest::Client::new()
            .post(&address)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let text = get_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, text }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut conf = get_configuration().expect("Unable to load configuration");
        let database_name = Uuid::new_v4().to_string();
        conf.database.database_name = database_name;
        conf.application.port = 0;
        conf.email_client.base_url = email_server.uri();
        conf
    };

    let db_pool: PgPool = configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Unable to build application");

    let port = application.port();
    let address = format!("http://{}:{}", &configuration.application.host, port);

    tokio::spawn(application.run_until_stopped());

    println!("Address is : {}", address);

    TestApp {
        addr: address,
        port: port,
        pool: configuration.database.get_connection_pool(),
        email_server: email_server,
    }
}

pub async fn configure_database(database: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&database.without_db())
        .await
        .expect("unable to connect to pg");

    println!("Database name {}", &database.database_name);
    connection
        .execute(format!(r#"create database "{}";"#, &database.database_name).as_str())
        .await
        .expect("Failed to create database");

    let db_pool = PgPool::connect_with(database.with_db())
        .await
        .expect("Failed to connect to PG");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Unablet to migrate the database");

    db_pool
}
