use crate::State;
use actix_web::client::Client;
use actix_web::{get, post, web, Responder};
use juz::{Block, BlockChain, Transaction, TransactionPartial};
use std::collections::BTreeMap;

#[get("blockchain/")]
pub async fn get_all(state: web::Data<State>) -> impl Responder {
    let blockchain = state.blockchain.lock().unwrap();
    web::Json(blockchain.to_owned())
}

#[post("transaction/")]
pub async fn add_transaction(
    state: web::Data<State>,
    data: web::Json<Transaction>,
) -> impl Responder {
    let data = data.into_inner();
    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.add_new_transaction(&data);
    web::Json(data)
}

#[post("transaction/broadcast/")]
pub async fn add_transaction_broadcast(
    state: web::Data<State>,
    data: web::Json<TransactionPartial>,
) -> impl Responder {
    let data: Transaction = (&data.into_inner()).into();
    let urls = state.nodes.lock().unwrap();

    for node in urls.keys() {
        let endpoint = format!("{}/transaction/", node);
        let response = Client::new().post(endpoint).send_json(&data).await;
        log::debug!("{:?}", response);
    }

    let mut blockchain = state.blockchain.lock().unwrap();
    blockchain.add_new_transaction(&data);
    web::Json(data)
}

#[get("mine/")]
pub async fn mine(state: web::Data<State>) -> impl Responder {
    let mut blockchain = state.blockchain.lock().unwrap();
    let urls = state.nodes.lock().unwrap();
    let uuid = state.uuid.to_owned();
    let nonce = blockchain.proof_of_work();
    let block = blockchain.create_new_block(nonce);

    // broadcast
    for node in urls.keys() {
        let endpoint = format!("{}/recieve-block/", node);
        let response = Client::new().post(endpoint).send_json(&block).await;
        log::debug!("{:?}", response);
    }

    blockchain.push_block(block.to_owned());

    // add the reward to the new_transactions
    let reward = Transaction::new_reward(&uuid);

    // and broadcast this to peers
    for node in urls.keys() {
        let endpoint = format!("{}/transaction/", node);
        let response = Client::new().post(endpoint).send_json(&reward).await;
        log::debug!("{:?}", response);
    }

    blockchain.new_transactions.push(reward);

    web::Json(block)
}

#[post("recieve-block/")]
pub async fn recieve_block(state: web::Data<State>, block: web::Json<Block>) -> impl Responder {
    let mut blockchain = state.blockchain.lock().unwrap();
    let block = block.into_inner();

    if blockchain.verify_block(&block) {
        for transaction in &block.transactions {
            if let Some(i) = blockchain
                .new_transactions
                .iter()
                .position(|t| t.id == transaction.id)
            {
                blockchain.new_transactions.remove(i);
            }
        }
    }

    blockchain.push_block(block.to_owned());

    web::Json(block)
}

#[get("me/")]
pub async fn me(state: web::Data<State>) -> impl Responder {
    let url = state.url.to_owned();
    let uuid = state.uuid.to_owned();
    web::Json((url, uuid))
}

#[get("nodes/")]
pub async fn nodes(state: web::Data<State>) -> impl Responder {
    let nodes: Vec<(String, String)> = state
        .nodes
        .lock()
        .unwrap()
        .iter()
        .map(|s| (s.0.to_owned(), s.1.to_owned()))
        .collect();

    web::Json(nodes)
}

#[post("register-and-broadcast-node/")]
pub async fn register_broadcast(
    state: web::Data<State>,
    node: web::Json<(String, String)>,
) -> impl Responder {
    let mut urls = state.nodes.lock().unwrap();

    let (url, nid) = node.into_inner();
    log::debug!("{}", &url);

    log::debug!("1. broadcast this url to other nodes");
    for node in urls.keys() {
        let client = Client::default();
        let endpoint = format!("{}/register-node/", node);
        let response = client.post(endpoint).send_json(&(&url, &nid)).await;
        log::debug!("1. result:{:?}", response);
    }

    log::debug!("2. send it's own nodes to to the new node_url");
    let endpoint = format!("{}/register-node-bulk/", &url);
    let mut node_list = urls.to_owned();
    node_list.insert(state.url.to_owned(), state.uuid.to_owned());
    let response = Client::new().post(endpoint).send_json(&node_list).await;

    log::debug!("2. result: {:?}", response);

    log::debug!("3. add the url to it's own nodes");
    urls.insert(url, nid);

    "done".to_string()
}

#[post("register-node/")]
pub async fn register_node(
    state: web::Data<State>,
    node: web::Json<(String, String)>,
) -> impl Responder {
    let (url, nid) = node.into_inner();

    let mut nds = state.nodes.lock().unwrap();
    log::info!("registered {} ({})", &nid, &url);
    nds.insert(url, nid);
    "ok".to_string()
}

#[post("register-node-bulk/")]
pub async fn register_node_bulk(
    state: web::Data<State>,
    node_urls: web::Json<BTreeMap<String, String>>,
) -> impl Responder {
    let mut urls = state.nodes.lock().unwrap();
    let mut node_urls = node_urls.into_inner();

    log::info!("registering {:?}", &node_urls);
    urls.append(&mut node_urls);

    "ok".to_string()
}

#[get("consensus/")]
pub async fn consensus(
    state: web::Data<State>,
    node_urls: web::Json<BTreeMap<String, String>>,
    )-> impl Responder {
   
    let peers = node_urls.into_inner();
    let mut blockchain = state.blockchain.lock().unwrap();

    let mut max_chain_length = blockchain.length(); 
    let mut max_block_chain: Option<BlockChain> = None;
    
    for url in peers.keys() {
        let endpoint = format!("{}/blockchain/", url);
        // FIXME: dangerous unwrapping
        let mut response = Client::new().get(endpoint).send().await.unwrap();
        let other_bc: BlockChain = response.json().await.unwrap(); 
        if other_bc.length() > max_chain_length {
            max_chain_length = other_bc.length();
            max_block_chain = Some(other_bc.to_owned());
        }
    }

    if max_chain_length != blockchain.length() {
        if let Some(bc) =  max_block_chain {
            blockchain.set_chain(bc.get_chain());
            blockchain.set_transactions(bc.new_transactions);
            return "replaced".to_string();
        }
    }

    "ok".to_string()
}
