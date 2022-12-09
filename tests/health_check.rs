use sqlx::PgPool;
use zero2prod::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use once_cell::sync::Lazy;
    use secrecy::ExposeSecret;
    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use uuid::Uuid;
    use zero2prod::{
        configuration::{get_configuration, DatabaseSettings},
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
    use super::*;

    #[tokio::test]
    async fn health_check_test() {
        let TestApp { address, db_pool } = spawn_app().await;
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/health_check", address))
            .send()
            .await
            .expect("Failed to execute request");

        assert!(response.status().is_success());
        assert_eq!(Some(0), response.content_length());
    }

    #[tokio::test]
    async fn subscribe_returns_200_for_valid_form_data() {
        let TestApp { address, db_pool } = spawn_app().await;
        let client = reqwest::Client::new();
        let body = "name=arun%20manivannan&email=arun%40arun.com";

        let response = client
            .post(format!("{}/subscribe", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(200, response.status().as_u16());

        let saved = sqlx::query!("SELECT email, name from subscriptions",)
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(saved.email, "arun@arun.com");
        assert_eq!(saved.name, "arun manivannan");
    }

    #[tokio::test]
    async fn subscribe_returns_400_for_missing_name_or_email() {
        let TestApp {
            address,
            db_pool: _,
        } = spawn_app().await;
        let client = reqwest::Client::new();
        let body = [
            ("name=arun%20manivannan", "missing email"),
            ("email=arun%40arun.com", "missing name"),
            ("", "missing both name and email"),
        ];

        for (each_body, each_err) in body {
            let response = client
                .post(format!("{}/subscribe", address))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(each_body)
                .send()
                .await
                .expect("Unable to execute request");

            println!("Response status:{}", response.status().as_u16());
            assert_eq!(
                400,
                response.status().as_u16(),
                "The API failed with 400 when the payload was {}",
                each_err
            );
        }
    }

    async fn spawn_app() -> TestApp {
        Lazy::force(&TRACING);
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind local port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);
        let mut configuration = get_configuration().expect("Failed to read configuration");
        let uuid_db_name = Uuid::new_v4().to_string();
        configuration.database.database_name = uuid_db_name;

        let db_pool = configure_database(&configuration.database).await;

        let server = run(listener, db_pool.clone()).expect("Failed to bind address");
        tokio::spawn(server);

        TestApp { address, db_pool }
    }

    async fn configure_database(db_settings: &DatabaseSettings) -> PgPool {
        //Create database
        let mut connection =
            PgConnection::connect(db_settings.connection_string_without_db().expose_secret())
                .await
                .expect("Unable to connect to Postgres");

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, db_settings.database_name).as_str())
            .await
            .expect("Failed to create databse");

        let db_pool = PgPool::connect(db_settings.connection_string().expose_secret())
            .await
            .expect("Failed to connect to postgres");

        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to migrate the database");

        db_pool
    }
}
