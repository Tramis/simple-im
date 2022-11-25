use std::time::Instant;

use actix::{Actor, Addr};
use actix_files::{Files, NamedFile};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

mod id;
mod server;
mod session;

use log::{error, info, warn};

async fn index() -> impl Responder {
    NamedFile::open_async("./public/index.html").await.unwrap()
}

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

fn cook_log() -> log4rs::Handle {
    let stdout = log4rs::append::console::ConsoleAppender::builder().build();
    let config = log4rs::Config::builder()
        .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            log4rs::config::Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _log_handler = cook_log();

    // start chat server actor
    let server = server::ChatServer::new().start();

    // at http://0.0.0.0:8080

    HttpServer::new(move || {
        App::new()
            .service(web::resource("/").to(index))
            .app_data(web::Data::new(server.clone()))
            .service(Files::new("/static", "./static"))
            .route("/ws", web::get().to(chat_route))
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
