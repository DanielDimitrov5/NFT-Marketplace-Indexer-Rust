use serde::{Serialize, Deserialize};
use ethers::types::H160;

#[derive(Serialize, Deserialize, Debug)]
pub struct NFTCollection {
    pub id: String,
    pub nft_collection: H160
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub item_id: u64, 
    pub nft_contract: H160,
    pub token_id: u64,
    pub owner: H160,
    pub price: u64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer{
    pub item_id: u64,
    pub offerer: H160,
    pub seller: H160,
    pub price: u64,
    pub is_accepted: bool
}

