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
use persistence::PersistenceModule;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub(crate) struct ProviderContext {
    pub cache: Arc<Cache>,
    pub persistence: Arc<PersistenceModule>,
}

type LoadFn = fn(&ProviderContext) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>>;

pub(crate) struct DataProvider {
    pub priority: u8,
    pub load: LoadFn,
}

inventory::collect!(DataProvider);

pub(crate) async fn load_all(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut providers: Vec<&DataProvider> = inventory::iter::<DataProvider>().collect();
    providers.sort_by_key(|p| p.priority);

    for provider in providers {
        (provider.load)(ctx).await?;
    }

    Ok(())
}
