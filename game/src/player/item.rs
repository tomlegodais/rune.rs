#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
    pub amount: u32,
}

impl Item {
    pub fn new(id: u16, amount: u32) -> Self {
        Self { id, amount }
    }
}

impl From<(u16, u32)> for Item {
    fn from((id, amount): (u16, u32)) -> Self {
        Self { id, amount }
    }
}

impl From<Item> for (u16, u32) {
    fn from(Item { id, amount }: Item) -> Self {
        (id, amount)
    }
}
