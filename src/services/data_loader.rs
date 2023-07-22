use ethers::prelude::*;
use ethers::providers::Provider;
use ethers::types::{Address, H160, U256};
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::models;
use crate::try_join_all;
use crate::Marketplace;

mod logging {
    pub use super::super::logging::logger;
}
use logging::logger::log_new_line;

pub mod initila_data {
    use super::*;

    type TaskResultCollections = Result<models::NFTCollection, Box<dyn Error + Send + Sync>>;
    type TaskCollections = JoinHandle<TaskResultCollections>;

    pub async fn get_all_collections(
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

    pub async fn get_all_items(
        contract: Arc<Marketplace<Provider<Http>>>,
    ) -> Result<Vec<models::Item>, Box<dyn Error + Send + Sync>> {
        let item_count = contract.item_count().await?;
        let item_count: usize = item_count.as_u64() as usize;

        let mut handles: Vec<TaskItems> = Vec::new();

        for i in 1..=item_count {
            let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
            let handle: JoinHandle<
                std::result::Result<models::Item, Box<dyn Error + Send + Sync>>,
            > = tokio::spawn(async move {
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

    type TaskResultOffers = Result<models::Offer, Box<dyn Error + Send + Sync>>;
    type TaskOffers = JoinHandle<TaskResultOffers>;

    async fn get_offers(
        contract: Arc<Marketplace<Provider<Http>>>,
        item_id: U256,
    ) -> Result<Vec<models::Offer>, Box<dyn Error + Send + Sync>> {
        let offerers: Vec<Address> = contract.get_offerers(item_id).await?;
        let offerers_set: HashSet<Address> = offerers.into_iter().collect();
        let unique_offerers: Vec<Address> = offerers_set.into_iter().collect();

        let mut handles: Vec<TaskOffers> = Vec::new();

        for offerer in unique_offerers {
            let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
            let handle: JoinHandle<
                std::result::Result<models::Offer, Box<dyn Error + Send + Sync>>,
            > = tokio::spawn(async move {
                match contract_clone.offers(item_id, offerer).await {
                    Ok((item_id, _nft_contract, _token_id, seller, price, is_accepted)) => {
                        let offer: models::Offer = models::Offer {
                            item_id: item_id.as_u64(),
                            offerer,
                            seller,
                            price: price.as_u64(),
                            is_accepted,
                        };

                        let output = format!("Offer {} added \n", item_id.as_u64());
                        log_new_line(&output)?;

                        Ok(offer)
                    }
                    Err(err) => Err(Box::new(err) as Box<dyn Error + Send + Sync>),
                }
            });

            handles.push(handle);
        }

        let results = try_join_all(handles).await?;

        let offers: Vec<models::Offer> = results.into_iter().map(|res| res.unwrap()).collect();

        Ok(offers)
    }

    pub async fn get_all_offers(
        contract: Arc<Marketplace<Provider<Http>>>,
    ) -> Result<Vec<Vec<models::Offer>>, Box<dyn Error + Send + Sync>> {
        let item_count = contract.item_count().await?;
        let item_count: usize = item_count.as_u64() as usize;

        let mut handles: Vec<JoinHandle<Result<Vec<models::Offer>, Box<dyn Error + Send + Sync>>>> =
            Vec::new();

        for i in 1..=item_count {
            let contract_clone: Arc<Marketplace<Provider<Http>>> = Arc::clone(&contract);
            let handle: JoinHandle<Result<Vec<models::Offer>, Box<dyn Error + Send + Sync>>> =
                tokio::spawn(async move { get_offers(contract_clone, U256::from(i)).await });

            handles.push(handle);
        }

        let results = try_join_all(handles).await?;

        let all_offers: Vec<Vec<models::Offer>> =
            results.into_iter().map(|res| res.unwrap()).collect();

        Ok(all_offers)
    }
}
