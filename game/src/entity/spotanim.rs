pub struct SpotAnim {
    pub id: u16,
    pub speed: u16,
    pub height: u16,
    pub rotation: u8,
}

impl SpotAnim {
    pub fn speed_height_hash(&self) -> u32 {
        (self.speed as u32 & 0xffff) | ((self.height as u32) << 16)
    }

    pub fn rotation_hash(&self) -> u8 {
        self.rotation & 0x7
    }
}

pub struct SpotAnimBuilder<F: FnOnce(SpotAnim)> {
    id: u16,
    speed: u16,
    height: u16,
    rotation: u8,
    apply: Option<F>,
}

impl<F: FnOnce(SpotAnim)> SpotAnimBuilder<F> {
    pub fn new(id: u16, apply: F) -> Self {
        Self {
            id,
            speed: 0,
            height: 0,
            rotation: 0,
            apply: Some(apply),
        }
    }

    pub fn speed(mut self, speed: u16) -> Self {
        self.speed = speed;
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn rotation(mut self, rotation: u8) -> Self {
        self.rotation = rotation;
        self
    }
}

impl<F: FnOnce(SpotAnim)> Drop for SpotAnimBuilder<F> {
    fn drop(&mut self) {
        if let Some(apply) = self.apply.take() {
            apply(SpotAnim {
                id: self.id,
                speed: self.speed,
                height: self.height,
                rotation: self.rotation,
            });
        }
    }
}
