use filesystem::loader::ItemLoader;
use filesystem::CacheBuilder;

fn main() -> anyhow::Result<()> {
    let cache = CacheBuilder::new("cache/").open()?;
    let items = ItemLoader::load(&cache)?;

    let coins = items.get(1040);
    println!("{:#?}", coins);

    Ok(())
}

