mod routes;

use actix_web::{App, HttpServer, web};
use routes::{hello, upload};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .service(hello::hello)
                    .service(upload::upload)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
