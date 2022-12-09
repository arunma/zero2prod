use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use std::io::Error;
use std::net::TcpListener;

use actix_web::dev::Server;

use crate::routes::{healthcheck, subscribe};

pub fn run(listener: TcpListener, _pool: PgPool) -> Result<Server, Error> {
    let pool = web::Data::new(_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(healthcheck))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
