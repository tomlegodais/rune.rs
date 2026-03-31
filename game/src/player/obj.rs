#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Obj {
    pub id: u16,
    pub amount: u32,
}

impl Obj {
    pub fn new(id: u16, amount: u32) -> Self {
        Self { id, amount }
    }
}

impl From<(u16, u32)> for Obj {
    fn from((id, amount): (u16, u32)) -> Self {
        Self { id, amount }
    }
}

impl From<Obj> for (u16, u32) {
    fn from(Obj { id, amount }: Obj) -> Self {
        (id, amount)
    }
}
