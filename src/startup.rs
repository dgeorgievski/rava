use crate::routes::{health_check, healthcat_version};
use actix_web::dev::Server;
use actix_web::{web, get, App, HttpServer, HttpResponse};
use std::net::TcpListener;
use actix_web::dev::{ServiceResponse, ServiceRequest};
use tracing_actix_web::{TracingLogger, DefaultRootSpanBuilder, RootSpanBuilder, Level};
use actix_web::Error;
use tracing::Span;

pub struct CustomLevelRootSpanBuilder;

impl RootSpanBuilder for CustomLevelRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let level = if request.path() == "/healthz" {
            Level::DEBUG
        } else {
            Level::INFO
        };
        tracing_actix_web::root_span!(level = level, request)
    }

    fn on_request_end<B: actix_web::body::MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}


#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Welcome to rava!")
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    
    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::<CustomLevelRootSpanBuilder>::new())
            .service(index)
            .service(healthcat_version)
            .route("/healthz", web::get().to(health_check))
            
    })
    .listen(listener)?
    .run();

    Ok(server)
}
