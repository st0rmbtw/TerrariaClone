#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub(crate) struct FRect {
    pub centerx: f32,
    pub centery: f32,
    pub width: f32,
    pub height: f32
}

impl FRect {
    pub const fn new(centerx: f32, centery: f32, width: f32, height: f32) -> Self {
        Self { centerx, centery, width, height }
    }

    pub fn intersects(&self, other: &FRect) -> bool {
        if self.width == 0.0 || self.height == 0.0 || other.width == 0.0 || other.height == 0.0 {
            return false;
        }

        let self_left = self.left();
        let self_right = self.right();
        let self_top = self.top();
        let self_bottom = self.bottom();

        let other_left = other.left();
        let other_right = other.right();
        let other_top = other.top();
        let other_bottom = other.bottom();

        return self_left < other_right &&
               self_top > other_bottom &&
               self_right > other_left &&
               self_bottom < other_top;
    }

    pub fn inside(&self, point: (f32, f32)) -> bool {
        let left = self.left();
        let right = self.right();
        let top = self.top();
        let bottom = self.bottom();

        point.0 > left && point.0 < right && point.1 > bottom && point.1 < top
    }

    pub fn left(&self) -> f32 {
        self.centerx - self.width / 2.
    }

    pub fn right(&self) -> f32 {
        self.centerx + self.width / 2.
    }

    pub fn top(&self) -> f32 {
        self.centery + self.height / 2.
    }

    pub fn bottom(&self) -> f32 {
        self.centery - self.height / 2.
    }
}