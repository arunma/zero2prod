pub mod domain;
pub mod email_client;

use std::{net::TcpListener, time::Duration};

use sqlx::postgres::PgPoolOptions;
use zero2prod::email_client::EmailClient;

use zero2prod::startup::Application;
use zero2prod::telemetry::init_subscriber;
use zero2prod::{configuration::get_configuration, telemetry::get_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let trace_subscriber =
        get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(trace_subscriber);

    let configuration = get_configuration().expect("Unable to load configuration");

    let application = Application::build(configuration).await?;
    let application_task = application.run_until_stopped();

    application_task.await?;
    //tokio::spawn(application_task);
    Ok(())
}
