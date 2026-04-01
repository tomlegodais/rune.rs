use filesystem::{FileId, IndexId};
use macros::data_provider;
use once_cell::sync::OnceCell;
use util::HuffmanTable;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<HuffmanTable> = OnceCell::new();

#[data_provider]
async fn load_huffman(ctx: &ProviderContext) -> anyhow::Result<()> {
    let archive = ctx
        .cache
        .find_archive(IndexId::HUFFMAN, "huffman")?
        .ok_or_else(|| anyhow::anyhow!("huffman archive not found"))?;

    let table = ctx.cache.read_file(IndexId::HUFFMAN, archive, FileId::new(0))?;
    INSTANCE.get_or_init(|| HuffmanTable::build(&table));

    Ok(())
}

pub fn decode_huffman(data: &[u8], text_len: usize) -> String {
    let table = INSTANCE.get().expect("huffman not initialized");
    table.decode(data, text_len)
}

pub fn encode_huffman(text: &str) -> Vec<u8> {
    let table = INSTANCE.get().expect("huffman not initialized");
    table.encode(text)
}
