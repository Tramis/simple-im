use std::{
    path::{self, Path},
    time::Instant,
};

use actix::{Actor, Addr};
use actix_files::{Files, NamedFile};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

mod id;
mod server;
mod session;
mod sql;

use clap::Parser;
use log::{error, info, warn};

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

fn cook_log() {
    let config_str = include_str!("../config/log.yaml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();
}

/// # arguments.
/// - specifying `-d` and `-p` is neccessary
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path of `index.html`.
    #[arg(short, long)]
    index: String,

    /// specify the static resource directory.
    /// such as `index.html`, `*.css` files
    #[arg(short, long)]
    directory: Vec<String>,

    /// specify the port to expose. default: 8080
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// specify record file path to record chat log. empty for no recording
    #[arg(short, long)]
    record: Option<String>,
}

use once_cell::sync::OnceCell;
static INDEX_PATH: OnceCell<String> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // config log
    cook_log();

    // get arguments
    let args = Args::parse();
    let mut dirs: Vec<(String, String)> = vec![];

    for dir in args.directory {
        let path = Path::new(&dir);
        if !path.is_dir() {
            panic!("invalid dir: \"{dir}\" specified by `-d` / `--directory`")
        }

        dirs.push((
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .to_string(),
            dir,
        ))
    }

    // index
    INDEX_PATH.get_or_init(|| args.index);
    async fn index() -> impl Responder {
        NamedFile::open_async(INDEX_PATH.get().unwrap())
            .await
            .unwrap()
    }

    // start chat server actor
    let server = server::ChatServer::new().start();

    // at http://0.0.0.0:8080

    HttpServer::new(move || {
        let mut app = App::new();

        app = app
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index));

        for (dir_name, real_dir_name) in &dirs {
            info!("### map [{real_dir_name}] to [{dir_name}]");
            app = app.service(Files::new(&format!("/{dir_name}"), &real_dir_name));
        }

        // add restful services here
        app.route("/ws", web::get().to(chat_route))
    })
    .workers(2)
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
}
