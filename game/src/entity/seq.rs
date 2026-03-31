pub struct Seq {
    pub id: u16,
    pub speed: u8,
}

pub struct SeqBuilder<F: FnOnce(Seq)> {
    id: u16,
    speed: u8,
    apply: Option<F>,
}

impl<F: FnOnce(Seq)> SeqBuilder<F> {
    pub fn new(id: u16, apply: F) -> Self {
        Self {
            id,
            speed: 0,
            apply: Some(apply),
        }
    }

    pub fn speed(mut self, speed: u8) -> Self {
        self.speed = speed;
        self
    }
}

impl<F: FnOnce(Seq)> Drop for SeqBuilder<F> {
    fn drop(&mut self) {
        if let Some(apply) = self.apply.take() {
            apply(Seq {
                id: self.id,
                speed: self.speed,
            });
        }
    }
}
