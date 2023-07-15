use mongodb::{Client, options::ClientOptions};
use std::error::Error;
use tokio;

mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    client_options.app_name = Some("rust_mongo_project".to_string());

    let client = Client::with_options(client_options)?;

    let db = client.database("nft-mp");

    let collection = db.collection::<models::Collection>("collections");

    let item_collection = db.collection::<models::Item>("items");

    let offer_collection = db.collection::<models::Offer>("offers");

    let _result = collection.insert_one(models::Collection {
        id: "1".to_string(),
        nft_collection: "0x1".to_string()
    }, None).await?;

    let _result = item_collection.insert_one(models::Item {
        item_id: 1,
        nft_contract: "0x1".to_string(),
        token_id: 1,
        owner: "0x1".to_string(),
        price: 1,
        name: Some("Item 1".to_string()),
        description: Some("Item 1 description".to_string()),
        image: Some("Item 1 image".to_string())
    }, None).await?;

    let _result = offer_collection.insert_one(models::Offer {
        item_id: 1,
        offerer: "0x2".to_string(),
        seller: "0x1".to_string(),
        price: 1,
        is_accepted: false
    }, None).await?;

    Ok(())
}
