use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, get, App, Responder, HttpServer, HttpRequest, HttpResponse};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to rava!")
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(index)
            .route("/healthz", web::get().to(health_check))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
