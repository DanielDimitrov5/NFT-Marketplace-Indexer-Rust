use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection {
    pub id: String,
    pub nft_collection: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub item_id: i32, // replacing 'id' from your schema to avoid conflict with MongoDB's default '_id'
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

