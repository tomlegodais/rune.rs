use filesystem::{Cache, FileId, IndexId};
use macros::data_provider;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use util::HuffmanTable;

static INSTANCE: OnceCell<HuffmanTable> = OnceCell::new();

#[data_provider]
pub fn load_huffman(cache: &Arc<Cache>) -> anyhow::Result<()> {
    let archive = cache
        .find_archive(IndexId::HUFFMAN, "huffman")?
        .ok_or_else(|| anyhow::anyhow!("huffman archive not found"))?;

    let table = cache.read_file(IndexId::HUFFMAN, archive, FileId::new(0))?;
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
