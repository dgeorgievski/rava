use actix_web::{web, 
    get, 
    middleware, 
    App, 
    HttpRequest, 
    HttpResponse, 
    HttpServer, 
    Responder};

#[get("/")]
async fn greet() -> HttpResponse  {
    HttpResponse::Ok().body("Hello, World!")
}

#[get("/{name}")]
async fn greet_name(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
        web::scope("/hello")
                    .service(greet)
                    .service(greet_name),
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
