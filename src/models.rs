use serde::{Serialize, Deserialize};
use ethers::types::H160;

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
    pub id: String,
    pub nft_collection: H160
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub item_id: i32, 
    pub nft_contract: String,
    pub token_id: i32,
    pub owner: String,
    pub price: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer{
    pub item_id: i32,
    pub offerer: String,
    pub seller: String,
    pub price: i32,
    pub is_accepted: bool
}

