use std::collections::HashMap;

use ethers::types::H160;
use serde::{Serialize, Deserialize};
use crate::amm::factory::Factory;

#[derive(Serialize, Deserialize)]
pub struct DiscoverFactoriesEntry {
  pub last_block: u64,
  pub factories: HashMap<H160, (Factory, u64)>,
}

pub struct DiscoverFactoriesStorage {
  path: String,
  entry: DiscoverFactoriesEntry,
}

impl Default for DiscoverFactoriesStorage {
  fn default() -> Self {
    Self {
      path: "factories.bin".to_string(),
      entry: DiscoverFactoriesEntry {
        last_block: 0,
        factories: HashMap::new(),
      },
    }
  }
}

impl DiscoverFactoriesStorage {
  pub fn new(path: &str, factories: HashMap<H160, (Factory, u64)>, last_block: u64) -> Self {
    Self {
      path: path.to_string(),
      entry: DiscoverFactoriesEntry {
        last_block,
        factories,
      },
    }
  }

  pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
    let storage = std::fs::read_to_string(path)?;
    let entry: DiscoverFactoriesEntry = serde_json::from_str(&storage)?;

    Ok(Self {
      path: path.to_string(),
      entry,
    })
  }

  pub fn load_or_default(path: &str) -> Self {
    match Self::load(path) {
      Ok(storage) => storage,
      Err(_) => Self {
        path: path.to_string(),
        ..Default::default()
      },
    }
  }

  pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
    let storage = serde_json::to_string(&self.entry)?;
    std::fs::write(&self.path, storage)?;

    Ok(())
  }

  pub fn get_factories(&self) -> Vec<(&Factory, u64)> {
    self.entry.factories.values().map(|(factory, amms)| (factory, *amms)).collect()
  }

  pub fn get_last_block(&self) -> u64 {
    self.entry.last_block
  }

  pub fn set_last_block(&mut self, last_block: u64) {
    self.entry.last_block = last_block;
  }

  pub fn add_factory(&mut self, address: H160, factory: Factory) {
    self.entry.factories.insert(address, (factory, 0));
  }

  pub fn inc_amms(&mut self, address: H160) -> bool {
    match self.entry.factories.get_mut(&address) {
      Some((_, amms)) => {
        *amms += 1;
        true
      },
      None => false,
    }
  } 
}
