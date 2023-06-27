use super::{
    deploy::Brc20Deploy, mint::Brc20Mint, transfer::Brc20Transfer, user_balance::UserBalance,
    ToDocument,
};
use bitcoin::Address;
use mongodb::bson::{doc, Bson, DateTime, Document};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Brc20Ticker {
    pub tick: String,
    pub limit: f64,
    pub max_supply: f64,
    pub total_minted: f64,
    pub decimals: u8,
    pub deploy: Brc20Deploy,
    pub mints: Vec<Brc20Mint>,
    pub transfers: Vec<Brc20Transfer>,
    pub balances: HashMap<Address, UserBalance>,
}

impl ToDocument for Brc20Ticker {
    fn to_document(&self) -> Document {
        doc! {
            "tick": self.get_ticker().clone(),
            "limit": self.limit,
            "max_supply": self.max_supply,
            "decimals": self.decimals as i64,
            "total_minted": self.total_minted,
            "block_height": self.deploy.block_height,
            "created_at": Bson::DateTime(DateTime::now())
        }
    }
}

impl Brc20Ticker {
    pub fn new(deploy: Brc20Deploy) -> Brc20Ticker {
        let tick = deploy.get_deploy_script().tick.to_lowercase().clone();
        let limit = deploy.get_limit();
        let max_supply = deploy.get_max_supply();
        let decimals = deploy.get_decimals();

        Brc20Ticker {
            tick,
            limit,
            max_supply,
            total_minted: 0.0,
            decimals,
            deploy,
            mints: Vec::new(),
            transfers: Vec::new(),
            balances: HashMap::new(),
        }
    }

    pub fn get_ticker(&self) -> String {
        self.tick.to_lowercase()
    }
}
