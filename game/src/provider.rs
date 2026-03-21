use filesystem::{Cache, FileId, IndexId};

pub struct ProviderContext<'a> {
    pub cache: &'a Cache,
}

pub trait DataProvider {
    fn load(&self, ctx: &ProviderContext) -> anyhow::Result<()>;
}

pub fn load(cache: &Cache) -> anyhow::Result<()> {
    let ctx = ProviderContext { cache };
    let providers: Vec<Box<dyn DataProvider>> = vec![Box::new(HuffmanProvider)];

    for provider in &providers {
        provider.load(&ctx)?;
    }

    Ok(())
}

struct HuffmanProvider;

impl DataProvider for HuffmanProvider {
    fn load(&self, ctx: &ProviderContext) -> anyhow::Result<()> {
        let archive = ctx
            .cache
            .find_archive(IndexId::HUFFMAN, "huffman")?
            .ok_or_else(|| anyhow::anyhow!("huffman archive not found"))?;

        let table = ctx
            .cache
            .read_file(IndexId::HUFFMAN, archive, FileId::new(0))?;

        util::Huffman::init(&table);
        Ok(())
    }
}