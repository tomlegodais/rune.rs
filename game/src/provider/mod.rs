mod collision;
mod enums;
mod hair;
mod huffman;
mod obj;
mod loc;
mod npc;
mod structs;
mod varbit;

use std::{future::Future, pin::Pin, sync::Arc};

pub use collision::get_collision;
pub use enums::get_enum_type;
use filesystem::Cache;
pub use hair::{get_hair_low, get_hair_mid};
pub use huffman::{decode_huffman, encode_huffman};
pub use obj::get_obj_type;
pub use loc::get_loc_type;
pub use npc::get_npc_type;
use persistence::PersistenceModule;
pub use structs::get_struct_type;
pub use varbit::get_varbit_type;

pub struct ProviderContext {
    pub cache: Arc<Cache>,
    pub persistence: Arc<PersistenceModule>,
}

type LoadFn = fn(&ProviderContext) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>>;

pub struct DataProvider {
    pub priority: u8,
    pub load: LoadFn,
}

inventory::collect!(DataProvider);

pub async fn load_all(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut providers: Vec<&DataProvider> = inventory::iter::<DataProvider>().collect();
    providers.sort_by_key(|p| p.priority);

    for provider in providers {
        (provider.load)(ctx).await?;
    }

    Ok(())
}
