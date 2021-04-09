use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// this will always the same
const GENESIS_HASH: &str = "36bf8006d73be65dceea9e4770ddb23dd90118460fa29b648409e31a6b06d183";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    index: u32,
    transactions: Vec<Transaction>,
}

impl BlockData {
    pub fn new(index: u32, transactions: &Vec<Transaction>) -> Self {
        Self {
            index,
            transactions: transactions.to_owned(),
        }
    }
}

fn hash_block(prev_hash: &str, nonce: u32, block_data: &BlockData) -> String {
    let block_data = serde_json::to_string(block_data).unwrap();
    let cat = format!("{}{}{}", prev_hash, block_data, nonce);
    format!("{:x}", Sha256::digest(cat.as_bytes()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    index: u32,
    timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    nonce: u32,
    hash: String,
    prev_hash: String,
}

impl Block {

    pub fn genesis() -> Self {

        let nonce = 100;
        let transactions = Vec::new();
        let prev_hash = "0".to_string();

        // if we want to change the GENESIS_HASH
        // this will be the right process
        // let genesis_data = BlockData::new(1, &transactions);
        // let hash = hash_block(&prev_hash, nonce, &genesis_data);

        Block {
            index: 1,
            timestamp: Utc::now(),
            transactions,
            nonce,
            hash: GENESIS_HASH.to_string(),
            prev_hash,
        }

    }

    pub fn block_data(&self) -> BlockData {
        BlockData {
            index: self.index,
            transactions: self.transactions.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionPartial {
    amount: f64,
    sender: String,
    recipient: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    amount: f64,
    sender: String,
    recipient: String,
}


impl Transaction {
    pub fn new_reward(id: &str) ->Self {
        let partial = TransactionPartial{
            amount: 12.5,
            sender: "00".into(),
            recipient: id.into()
        };

        (&partial).into()
    }
}

impl From<TransactionPartial> for Transaction {
    fn from(partial: TransactionPartial) -> Self {
        let id: String = Uuid::new_v4()
            .to_simple()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_string();
        Self {
            id,
            amount: partial.amount,
            sender: partial.sender,
            recipient: partial.recipient,
        }
    }
}


impl From<&TransactionPartial> for Transaction {
    fn from(partial: &TransactionPartial) -> Self {
        let id: String = Uuid::new_v4()
            .to_simple()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_string();
        Self {
            id,
            amount: partial.amount,
            sender: partial.sender.to_string(),
            recipient: partial.recipient.to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BlockChain {
    chain: Vec<Block>,
    pub new_transactions: Vec<Transaction>,
}

impl BlockChain {
    pub fn new() -> Self {
        let mut bc = Self::default();
        
        let genesis = Block::genesis();
        bc.push_block(genesis);
        log::info!("blockchain generated");

        bc
    }

    pub fn length(&self) -> usize {
       self.chain.len() 
    }

    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.to_owned()
    }

    pub fn set_chain(&mut self, chain: Vec<Block>) {
        self.chain = chain;
    }

    pub fn set_transactions(&mut self, transactions: Vec<Transaction>) {
        self.new_transactions = transactions;
    }

    pub fn verify_block(&self, block: &Block) -> bool {
        let last_block = self.get_last_block();

        if block.prev_hash != last_block.hash {
            return false;
        }

        let block_data = BlockData::new(self.chain.len() as u32 + 1, &block.transactions);
        let hash = hash_block(&last_block.hash, block.nonce, &block_data);
        if hash != block.hash {
            return false;
        }

        true
    }

    pub fn verify(&self) -> bool {

        // check if the hashes match
        for i in 1..self.chain.len() {
            let block = &self.chain[i];
            let p_block = &self.chain[i-1];
            let block_data = block.block_data();
            let hash = hash_block(&p_block.hash, block.nonce, &block_data);

            if block.prev_hash != p_block.hash {
                return false;
            }

            if hash != block.hash {
                return false
            }
        }
        
        if self.chain[0] != Block::genesis() {
            return false
        }

        true 
    }

    pub fn create_new_block(&self, nonce: u32) -> Block {
        let index: u32 = (self.chain.len() + 1) as u32;
        let timestamp = Utc::now();
        let transactions = self.new_transactions.to_owned();

        let block_data = BlockData::new(index, &transactions);
        let last_block = self.get_last_block();
        let hash = hash_block(&last_block.hash, nonce, &block_data);

        Block {
            index,
            timestamp,
            transactions,
            nonce,
            hash,
            prev_hash: last_block.hash.to_owned(),
        }
    }

    pub fn push_block(&mut self, block: Block) {
        self.chain.push(block);
        self.new_transactions.clear();
    }

    pub fn get_last_block(&self) -> &Block {
        self.chain.last().unwrap() // we know this is true because we will have a genesis block
    }

    pub fn add_new_transaction(&mut self, transaction: &Transaction) {
        let tx = transaction.to_owned();
        self.new_transactions.push(tx);
    }

    pub fn proof_of_work(&self) -> u32 {
        let mut nonce = 0;
        let last_hash = self.get_last_block().hash.to_owned();
        let block_data = BlockData::new(self.chain.len() as u32 + 1, &self.new_transactions);

        loop {
            let hash = hash_block(&last_hash, nonce, &block_data);
            if hash.starts_with("0000") {
                break;
            }
            nonce += 1;
        }

        log::info!("nonce:{}", nonce);

        nonce
    }
}

#[cfg(test)]
mod bc_test {
    use super::*;

    #[test]
    fn test_new_block() {
        let mut bc = BlockChain::new();
        let block = bc.create_new_block(0);
        bc.push_block(block.to_owned());
        assert_eq!(block.index, bc.chain.len() as u32);
        assert_eq!(&block, bc.get_last_block());
    }

    #[test]
    fn test_new_transactions() {
        let mut bc = BlockChain::new();
        assert_eq!(bc.new_transactions.len(), 0);
        let partial = TransactionPartial {
            amount: 10.0,
            sender: "you".to_string(),
            recipient: "me".to_string(),
        };

        let transaction = (&partial).into();
        bc.add_new_transaction(&transaction);
        assert_eq!(bc.new_transactions.len(), 1);
        let block = bc.create_new_block(0);
        bc.push_block(block);
        assert_eq!(bc.new_transactions.len(), 0);
    }

    #[test]
    fn test_hash_block() {
        let mut bc = BlockChain::new();
        let block_1 = bc.create_new_block(0);
        bc.push_block(block_1.to_owned());
        let block_2 = bc.create_new_block(0);
        assert_ne!(block_1.hash, block_2.hash);
    }

    #[test]
    fn test_proof_of_work() {
        let mut bc = BlockChain::new();
        let partial = TransactionPartial {
            amount: 10.0,
            sender: "you".to_string(),
            recipient: "me".to_string(),
        };
        let transaction = &partial.into();
        bc.add_new_transaction(&transaction);
        let partial = TransactionPartial {
            amount: 20.0,
            sender: "me".to_string(),
            recipient: "you".to_string(),
        };
        let transaction = &partial.into();
        bc.add_new_transaction(&transaction);
        let nonce = bc.proof_of_work();
        let prev_hash = bc.get_last_block().hash.to_string();
        let block_data = BlockData::new(bc.chain.len() as u32 + 1, &bc.new_transactions);
        let hash = hash_block(&prev_hash, nonce, &block_data);
        assert!(hash.starts_with("0000"));
    }

    #[test]
    fn test_genesis_block() {
        let bc = BlockChain::new();
        let genesis = bc.get_last_block();
        assert_eq!(genesis.prev_hash, "0");
        assert_eq!(genesis.hash, GENESIS_HASH);
        assert_eq!(genesis.nonce, 100);
    }
}
