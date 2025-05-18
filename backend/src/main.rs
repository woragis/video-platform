use actix_web::{App, HttpServer};
use tokio_postgres::NoTls;
use actix_web::web::Data;

mod routes;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (db_client, connection) = tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=video_app", NoTls).await.unwrap();

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let db_data = Data::new(db_client);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(routes::video::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
