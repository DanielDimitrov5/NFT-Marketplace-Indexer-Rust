use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use std::env;

use ethers::prelude::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::Address;
use eyre::Result;
use futures::future::try_join_all;
use mongodb::{
    options::ClientOptions,
    Client,
};

mod services {
    pub mod data_loader;
    pub mod logging;
    pub mod listener;
}

use services::data_loader::initila_data::{get_all_collections, get_all_items,get_all_offers};
use services::listener::listening::listen_for_events;

mod models;

abigen!(Marketplace, "src\\data\\nftMartkeplace.json");
const CONTRACT_ADDRESS: &str = "0xf4351BA9Ca701Cf689442833CDA5F7FF18C2e00C";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();

    let rpc_url: String = env::var("RPC_URL").expect("RPC_URL must be set in a .env file");

    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    client_options.app_name = Some("rust_mongo_project".to_string());
    let client = Client::with_options(client_options)?;
    let db = client.database("nft-mp");

    File::create("src\\logs\\log.txt")?;

    let collection: mongodb::Collection<models::NFTCollection> =
        db.collection::<models::NFTCollection>("collections");
    collection.drop(None).await?;

    let item: mongodb::Collection<models::Item> = db.collection::<models::Item>("items");
    item.drop(None).await?;

    let offer: mongodb::Collection<models::Offer> = db.collection::<models::Offer>("offers");
    offer.drop(None).await?;

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client: Arc<_> = Arc::new(provider);
    let address: Address = CONTRACT_ADDRESS.parse()?;
    let contract: Arc<Marketplace<_>> = Arc::new(Marketplace::new(address, client));

    let collections: Vec<models::NFTCollection> = get_all_collections(contract.clone()).await?;
    collection.insert_many(collections, None).await?;

    let items: Vec<models::Item> = get_all_items(contract.clone()).await?;
    item.insert_many(items, None).await?;

    let all_offers: Vec<Vec<models::Offer>> = get_all_offers(contract.clone()).await?;

    for offet_vec in all_offers {
        if offet_vec.len() == 0 {
            continue;
        }
        offer.insert_many(offet_vec, None).await?;
    }

    listen_for_events(&contract, &collection, &item, &offer).await?;

    Ok(())
}
