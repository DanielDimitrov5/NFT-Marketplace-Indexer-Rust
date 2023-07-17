use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;

use eyre::Result;
use ethers::prelude::abigen;
use ethers::providers::{Http, Provider, StreamExt};
use ethers::types::{Address, H160, U256};
use futures::future::try_join_all;
use hex::encode;
use mongodb::{
    bson::{doc, to_bson},
    options::ClientOptions,
    Client,
};
use tokio::task::JoinHandle;

mod models;

abigen!(Marketplace, "src\\data\\nftMartkeplace.json");
const RPC_URL: &str = "https://sepolia.infura.io/v3/09755767452a49d3a5b3f9b84d9db6c9";
// const CONTRACT_ADDRESS: &str = "0xf4351BA9Ca701Cf689442833CDA5F7FF18C2e00C";
const CONTRACT_ADDRESS: &str = "0x411F60BB2345C7FD0572CE8F10252D45b691F27c";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let client: Arc<_> = Arc::new(provider);
    let address: Address = CONTRACT_ADDRESS.parse()?;
    let contract: Arc<Marketplace<_>> = Arc::new(Marketplace::new(address, client));

    let collections = get_all_collections(contract.clone()).await?;
    collection.insert_many(collections, None).await?;

    let items = get_all_items(contract.clone()).await?;
    item.insert_many(items, None).await?;

    listen_for_events(&contract, &collection, &item).await?;

    Ok(())
}

async fn listen_for_events(
    contract: &Marketplace<Provider<Http>>,
    collection: &mongodb::Collection<models::NFTCollection>,
    item: &mongodb::Collection<models::Item>,
) -> Result<()> {
    let events = contract.events();
    let mut stream = events.stream().await?;

    while let Some(Ok(evt)) = stream.next().await {
        match evt {
            MarketplaceEvents::LogCollectionAddedFilter(event) => {
                let collection_model: models::NFTCollection = models::NFTCollection {
                    id: event.id.to_string(),
                    nft_collection: event.nft_collection,
                };

                collection.insert_one(collection_model, None).await?;

                let output = format!("Collection added: {:?}", event);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogItemListedFilter(event) => {
                let id = event.id.as_u64();
                let id_bson = to_bson(&id)?;

                let price = event.price.as_u64();
                let price_bson = to_bson(&price)?;

                item.update_one(
                    doc! {"item_id": id_bson},
                    doc! {"$set": {"price": price_bson}},
                    None,
                )
                .await?;

                let output = format!("Item {} listed for {} wei", event.id, price);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogItemAddedFilter(event) => {

                let item_model: models::Item = models::Item {
                    item_id: event.id.as_u64(),
                    nft_contract: event.nft_contract,
                    token_id: event.token_id.as_u64(),
                    owner: event.owner,
                    price: 0,
                    name: None,
                    description: None,
                    image: None,
                };

                item.insert_one(item_model, None).await?;

                let output = format!("Item added: {:?}", event);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogItemSoldFilter(event) => {
                let id = event.id.as_u64();
                let id_bson = to_bson(&id)?;

                let buyer: String = format!("0x{}", encode(event.buyer.as_fixed_bytes()));

                item.update_one(
                    doc! {"item_id": id_bson},
                    doc! {"$set": {"owner": &buyer, "price": 0}},
                    None,
                ).await?;

                let output = format!("Item {} sold to {}", event.id, &buyer);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogItemClaimedFilter(event) => {
                let id = event.id.as_u64();
                let id_bson = to_bson(&id)?;

                let owner: String = format!("0x{}", encode(event.claimer.as_fixed_bytes()));

                item.update_one(
                    doc! {"item_id": id_bson},
                    doc! {"$set": {"owner": &owner}},
                    None,
                ).await?;

                let output = format!("Item {} claimed by {}", event.id, &owner);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogOfferAcceptedFilter(event) => {
                let output = format!("Offer accepted: {:?}", event);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::LogOfferPlacedFilter(event) => {
                let output = format!("Offer placed: {:?}", event);

                log_new_line(&output)?;

                println!("{}", output);
            }
            MarketplaceEvents::OwnershipTransferredFilter(event) => {
                let output = format!("Ownership trnasferd: {:?}", event);

                log_new_line(&output)?;

                println!("{}", output);
            }
        }
    }

    Ok(())
}

type TaskResultCollections = Result<models::NFTCollection, Box<dyn Error + Send + Sync>>;
type TaskCollections = JoinHandle<TaskResultCollections>;

async fn get_all_collections(
    contract: Arc<Marketplace<Provider<Http>>>,
) -> Result<Vec<models::NFTCollection>, Box<dyn Error + Send + Sync>> {
    let collection_count = contract.collection_count().await?;
    let collection_count: usize = collection_count.as_u64() as usize;

    let mut handles: Vec<TaskCollections> = Vec::new();

    for i in 1..=collection_count {
        let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
        let handle: JoinHandle<
            std::result::Result<models::NFTCollection, Box<dyn Error + Send + Sync>>,
        > = tokio::spawn(async move {
            let collection: H160 = contract_clone.collections(U256::from(i)).await?;
            let collection_id: usize = i;
            let collection_address: H160 = collection;

            let collection: models::NFTCollection = models::NFTCollection {
                id: collection_id.to_string(),
                nft_collection: collection_address,
            };

            let output = format!("Collection {} of {} fetched \n", i, collection_count);

            log_new_line(&output)?;

            Ok(collection)
        });

        handles.push(handle);
    }

    let results = try_join_all(handles).await?;

    let collections: Vec<models::NFTCollection> =
        results.into_iter().map(|res| res.unwrap()).collect();

    Ok(collections)
}

type TaskResultItems = Result<models::Item, Box<dyn Error + Send + Sync>>;
type TaskItems = JoinHandle<TaskResultItems>;

async fn get_all_items(
    contract: Arc<Marketplace<Provider<Http>>>,
) -> Result<Vec<models::Item>, Box<dyn Error + Send + Sync>> {
    let item_count = contract.item_count().await?;
    let item_count: usize = item_count.as_u64() as usize;

    let mut handles: Vec<TaskItems> = Vec::new();

    for i in 1..=item_count {
        let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
        let handle: JoinHandle<std::result::Result<models::Item, Box<dyn Error + Send + Sync>>> =
            tokio::spawn(async move {
                let (item_id, nft_contract, token_id, owner, price): (
                    U256,
                    H160,
                    U256,
                    H160,
                    U256,
                ) = contract_clone.items(U256::from(i)).await?;

                let item: models::Item = models::Item {
                    item_id: item_id.as_u64(),
                    nft_contract,
                    token_id: token_id.as_u64(),
                    owner,
                    price: price.as_u64(),
                    name: None,
                    description: None,
                    image: None,
                };

                let output = format!("Item {} added \n", item_id.as_u64());

                log_new_line(&output)?;

                Ok(item)
            });

        handles.push(handle);
    }

    let results = try_join_all(handles).await?;

    let items: Vec<models::Item> = results.into_iter().map(|res| res.unwrap()).collect();

    Ok(items)
}

fn log_new_line(line: &str) -> std::io::Result<()> {
    let path = Path::new("src/logs/log.txt");
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&path)?;

    write!(file, "{}", line)?;
    Ok(())
}