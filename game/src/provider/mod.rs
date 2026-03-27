mod collision;
mod huffman;
mod item;
mod loc;
mod npc;
mod varbit;

pub use collision::get_collision;
pub use huffman::{decode_huffman, encode_huffman};
pub use item::get_item_definition;
pub use loc::get_loc_definition;
pub use npc::get_npc_definition;
pub use varbit::get_varbit_definition;

use filesystem::Cache;
use std::sync::Arc;

struct DataProvider {
    pub priority: u8,
    pub load: fn(&Arc<Cache>) -> anyhow::Result<()>,
}

inventory::collect!(DataProvider);

pub fn load_all(cache: &Arc<Cache>) -> anyhow::Result<()> {
    let mut providers: Vec<&DataProvider> = inventory::iter::<DataProvider>().collect();
    providers.sort_by_key(|p| p.priority);

    for provider in providers {
        (provider.load)(cache)?;
    }

    Ok(())
}
