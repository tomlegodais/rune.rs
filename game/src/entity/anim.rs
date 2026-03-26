pub struct Anim {
    pub id: u16,
    pub speed: u8,
}

pub struct AnimBuilder<F: FnOnce(Anim)> {
    id: u16,
    speed: u8,
    apply: Option<F>,
}

impl<F: FnOnce(Anim)> AnimBuilder<F> {
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

impl<F: FnOnce(Anim)> Drop for AnimBuilder<F> {
    fn drop(&mut self) {
        if let Some(apply) = self.apply.take() {
            apply(Anim {
                id: self.id,
                speed: self.speed,
            });
        }
    }
}
