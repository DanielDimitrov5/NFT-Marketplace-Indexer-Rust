use ethers::providers::{Http, Provider, StreamExt};
use eyre::Result;
use hex::encode;
use mongodb::bson::{doc, to_bson};

mod logging {
    pub use super::super::logging::logger;
}
use logging::logger::log_new_line;

use crate::models;
use crate::Marketplace;
use crate::MarketplaceEvents;

pub mod listening {
    use super::*;

    pub async fn listen_for_events(
        contract: &Marketplace<Provider<Http>>,
        collection: &mongodb::Collection<models::NFTCollection>,
        item: &mongodb::Collection<models::Item>,
        offer: &mongodb::Collection<models::Offer>,
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

                    let output = format!("Collection added: {:?} \n", event);

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

                    let id_bson = to_bson(&id)?;

                    offer.delete_many(doc! {"item_id": id_bson}, None).await?;

                    let output = format!("Item {} listed for {} wei \n", event.id, price);

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
                    )
                    .await?;

                    let output = format!("Item {} sold to {} \n", event.id, &buyer);

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
                    )
                    .await?;

                    let id_bson = to_bson(&id)?;

                    offer.delete_many(doc! {"item_id": id_bson}, None).await?;

                    let output = format!("Item {} claimed by {} \n", event.id, &owner);

                    log_new_line(&output)?;

                    println!("{}", output);
                }
                MarketplaceEvents::LogOfferAcceptedFilter(event) => {
                    let id = event.id.as_u64();
                    let id_bson = to_bson(&id)?;

                    let find_offer = doc! {"item_id": id_bson, "offerer": format!("0x{}", encode(event.offerer.as_fixed_bytes()))};

                    let find_offer = offer.find_one(find_offer, None).await?;

                    if let Some(_find_offer) = find_offer {
                        let item_id = event.id.as_u64();
                        let item_id_bson = to_bson(&item_id)?;

                        offer.update_one(
                            doc! {"item_id": item_id_bson, "offerer": format!("0x{}", encode(event.offerer.as_fixed_bytes()))},
                            doc! {"$set": {"is_accepted": true}},
                            None,
                        ).await?;

                        let output = format!("Offer {} accepted \n", event.id);

                        log_new_line(&output)?;

                        println!("{}", output);
                    }
                }
                MarketplaceEvents::LogOfferPlacedFilter(event) => {
                    let id = event.id.as_u64();
                    let id_bson = to_bson(&id)?;

                    let seller = contract.items(event.id).await?.3;

                    let find_offer = doc! {"item_id": id_bson, "offerer": format!("0x{}", encode(event.buyer.as_fixed_bytes())), "seller": format!("0x{}", encode(seller.as_fixed_bytes()))};

                    let find_offer = offer.find_one(find_offer, None).await?;

                    if let Some(_find_offer) = find_offer {
                        let item_id = event.id.as_u64();
                        let item_id_bson = to_bson(&item_id)?;

                        let price = event.price.as_u64();
                        let price_bson = to_bson(&price)?;

                        offer.update_one(
                            doc! {"item_id": item_id_bson, "offerer": format!("0x{}", encode(event.buyer.as_fixed_bytes())), "seller": format!("0x{}", encode(seller.as_fixed_bytes()))},
                            doc! {"$set": {"price": price_bson, "is_accepted": false}},
                            None,
                        ).await?;
                    } else {
                        let offer_model: models::Offer = models::Offer {
                            item_id: event.id.as_u64(),
                            offerer: event.buyer,
                            seller,
                            price: event.price.as_u64(),
                            is_accepted: false,
                        };

                        offer.insert_one(offer_model, None).await?;
                    }

                    let output = format!("Offer placed: {:?} \n", event);

                    log_new_line(&output)?;

                    println!("{}", output);
                }
                MarketplaceEvents::OwnershipTransferredFilter(event) => {
                    let output = format!("Ownership trnasferd: {:?} \n", event);

                    log_new_line(&output)?;

                    println!("{}", output);
                }
            }
        }

        Ok(())
    }
}
