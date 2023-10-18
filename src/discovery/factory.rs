use std::sync::Arc;

use ethers::{
    providers::Middleware,
    types::{Filter, H256},
};
// use serde::{Deserialize, Serialize};
// use spinoff::{spinners, Color, Spinner};

use crate::{
    amm::{self, factory::Factory},
    errors::AMMError,
};

use super::storage::DiscoverFactoriesStorage;

pub enum DiscoverableFactory {
    UniswapV2Factory,
    UniswapV3Factory,
}

impl DiscoverableFactory {
    pub fn discovery_event_signature(&self) -> H256 {
        match self {
            DiscoverableFactory::UniswapV2Factory => {
                amm::uniswap_v2::factory::PAIR_CREATED_EVENT_SIGNATURE
            }

            DiscoverableFactory::UniswapV3Factory => {
                amm::uniswap_v3::factory::POOL_CREATED_EVENT_SIGNATURE
            }
        }
    }
}

// Returns a vec of empty factories that match one of the Factory interfaces specified by each DiscoverableFactory
pub async fn discover_factories<M: Middleware>(
    factories: Vec<DiscoverableFactory>,
    number_of_amms_threshold: u64,
    middleware: Arc<M>,
    step: u64,
    storage_path: &str,
) -> Result<Vec<Factory>, AMMError<M>> {

    let mut storage = DiscoverFactoriesStorage::load_or_default(storage_path);
    let mut event_signatures = vec![];

    for factory in factories {
        event_signatures.push(factory.discovery_event_signature());
    }

    let block_filter = Filter::new().topic0(event_signatures);

    let mut from_block = storage.get_last_block();

    println!("from_block: {}", from_block);

    let current_block = middleware
        .get_block_number()
        .await
        .map_err(AMMError::MiddlewareError)?
        .as_u64();

    println!("current_block: {}", current_block);

    //For each block within the range, get all pairs asynchronously
    // let step = 100000;

    //Set up filter and events to filter each block you are searching by
    // let mut identified_factories: HashMap<H160, (Factory, u64)> = HashMap::new();

    // for factory in storage.get_factories() {
    //     identified_factories.insert(factory.address(), (factory.clone(), 0));
    // }

    //TODO: make this async
    while from_block < current_block {
        //Get pair created event logs within the block range
        let mut target_block = from_block + step - 1;
        if target_block > current_block {
            target_block = current_block;
        }

        let block_filter = block_filter.clone();

        let logs = match middleware
            .get_logs(&block_filter.from_block(from_block).to_block(target_block))
            .await {
            Ok(logs) => logs,
            Err(err) => {
                // spinner.fail("Error when getting logs");
                println!("Error when getting logs: {}", err);
                return Err(AMMError::MiddlewareError(err));
            }
      };

        for log in logs {
            if !storage.inc_amms(log.address) {
                let mut factory = Factory::try_from(log.topics[0])?;

                match &mut factory {
                    Factory::UniswapV2Factory(uniswap_v2_factory) => {
                        uniswap_v2_factory.address = log.address;
                        uniswap_v2_factory.creation_block = log
                            .block_number
                            .ok_or(AMMError::BlockNumberNotFound)?
                            .as_u64();
                    }
                    Factory::UniswapV3Factory(uniswap_v3_factory) => {
                        uniswap_v3_factory.address = log.address;
                        uniswap_v3_factory.creation_block = log
                            .block_number
                            .ok_or(AMMError::BlockNumberNotFound)?
                            .as_u64();
                    }
                }


                // identified_factories.insert(log.address, (factory.clone(), 0));
                
                storage.add_factory(log.address, factory);
                storage.save().map_err(|_| AMMError::StorageError)?;
            }
        }

        from_block += step;

        // update block
        storage.set_last_block(from_block);
        storage.save().map_err(|_| AMMError::StorageError)?;
    }

		storage.set_last_block(current_block);
		storage.save().map_err(|_| AMMError::StorageError)?;

    let mut filtered_factories = vec![];

    for (factory, amms_length) in storage.get_factories() {
        if amms_length >= number_of_amms_threshold {
            filtered_factories.push(factory.clone());
        }
    }

    // spinner.success("All factories discovered");
    Ok(filtered_factories)
}
