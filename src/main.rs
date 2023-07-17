use std::error::Error;
use std::sync::Arc;

use eyre::Result;
use mongodb::{options::ClientOptions, Client};

use ethers::prelude::abigen;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, H160, U256};

use futures::future::try_join_all;

use tokio::task::JoinHandle;

mod models;

abigen!(Marketplace, "src\\data\\nftMartkeplace.json");
const RPC_URL: &str = "https://sepolia.infura.io/v3/09755767452a49d3a5b3f9b84d9db6c9";
const CONTRACT_ADDRESS: &str = "0xf4351BA9Ca701Cf689442833CDA5F7FF18C2e00C";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    client_options.app_name = Some("rust_mongo_project".to_string());
    let client = Client::with_options(client_options)?;
    let db = client.database("nft-mp");

    let collection = db.collection::<models::Collection>("collections");
    collection.drop(None).await?;

    let item = db.collection::<models::Item>("items");
    item.drop(None).await?;

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let client: Arc<_> = Arc::new(provider);
    let address: Address = CONTRACT_ADDRESS.parse()?;
    let contract: Arc<Marketplace<_>> = Arc::new(Marketplace::new(address, client));

    let collections = get_all_collections(contract.clone()).await?;
    collection.insert_many(collections, None).await?;
    
    let items = get_all_items(contract.clone()).await?;
    item.insert_many(items, None).await?;
    

    Ok(())
}

type TaskResultCollections = Result<models::Collection, Box<dyn Error + Send + Sync>>;
type TaskCollections = JoinHandle<TaskResultCollections>;

async fn get_all_collections(
    contract: Arc<Marketplace<Provider<Http>>>,
) -> Result<Vec<models::Collection>, Box<dyn Error + Send + Sync>> {
    let collection_count = contract.collection_count().await?;
    let collection_count: usize = collection_count.as_u64() as usize;

    let mut handles: Vec<TaskCollections> = Vec::new();

    for i in 1..=collection_count {
        let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
        let handle: JoinHandle<
            std::result::Result<models::Collection, Box<dyn Error + Send + Sync>>,
        > = tokio::spawn(async move {
            let collection: H160 = contract_clone.collections(U256::from(i)).await?;
            let collection_id: usize = i;
            let collection_address: H160 = collection;

            let collection: models::Collection = models::Collection {
                id: collection_id.to_string(),
                nft_collection: collection_address,
            };

            Ok(collection)
        });

        handles.push(handle);
    }

    let results = try_join_all(handles).await?;

    let collections: Vec<models::Collection> =
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

                Ok(item)
            });

        handles.push(handle);
    }

    let results = try_join_all(handles).await?;

    let items: Vec<models::Item> = results.into_iter().map(|res| res.unwrap()).collect();

    Ok(items)
}
