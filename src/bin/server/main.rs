mod handlers;

use actix_web::{web, App, HttpServer};
use handlers::{
    add_transaction, add_transaction_broadcast, consensus, get_all, me, mine, nodes, recieve_block,
    register_broadcast, register_node, register_node_bulk,
};
use juz::BlockChain;
use std::collections::BTreeMap;
use std::env;
use std::sync::Mutex;
use uuid::Uuid;

pub struct State {
    blockchain: Mutex<BlockChain>,
    uuid: String,
    url: String,
    nodes: Mutex<BTreeMap<String, String>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    env::set_var("RUST_LOG", "actix_web=info,server=debug,juz=debug");
    env_logger::init();

    let port = args
        .get(1)
        .and_then(|s| Some(s.parse::<u16>().unwrap()))
        .or(Some(8880))
        .unwrap();

    let url = args
        .get(2)
        .map(|u| u.to_string())
        .or(Some("localhost:8880".into()))
        .unwrap();

    let uuid = Uuid::new_v4().to_simple().to_string();

    let state = web::Data::new(State {
        blockchain: Mutex::new(BlockChain::new()),
        uuid,
        url,
        nodes: Mutex::new(BTreeMap::new()),
    });

    log::info!("server starting");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(me)
            .service(nodes)
            .service(get_all)
            .service(add_transaction)
            .service(add_transaction_broadcast)
            .service(mine)
            .service(recieve_block)
            .service(register_broadcast)
            .service(register_node_bulk)
            .service(register_node)
            .service(consensus)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
