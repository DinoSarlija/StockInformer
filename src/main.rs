#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate validator;

use actix_web::{http, middleware::Logger, web, App, HttpServer};

pub mod infrastructure;
pub mod models;
pub mod schema;

use actix_cors::Cors;
fn setup_cors() -> Cors {
    Cors::default()
        .send_wildcard()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(infrastructure::state::initialize()))
            .wrap(Logger::default())
            .app_data(web::JsonConfig::default().limit(4096))
            .wrap(setup_cors())
            .service(web::scope("/").configure(infrastructure::routes::setup_routes))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
