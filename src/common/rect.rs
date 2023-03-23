#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub(crate) struct FRect {
    pub centerx: f32,
    pub centery: f32,
    pub width: f32,
    pub height: f32
}

impl FRect {
    pub(crate) const fn new(centerx: f32, centery: f32, width: f32, height: f32) -> Self {
        Self { centerx, centery, width, height }
    }

    pub(crate) fn intersects(&self, other: &FRect) -> bool {
        if self.width == 0.0 || self.height == 0.0 || other.width == 0.0 || other.height == 0.0 {
            return false;
        }

        return self.left() <= other.right() &&
               self.top() >= other.bottom() &&
               self.right() >= other.left() &&
               self.bottom() <= other.top();
    }

    pub(crate) fn inside(&self, point: (f32, f32)) -> bool {
        point.0 > self.left() && point.0 < self.right() && point.1 > self.bottom() && point.1 < self.top()
    }

    pub(crate) fn left(&self) -> f32 {
        self.centerx - self.width / 2.
    }

    pub(crate) fn right(&self) -> f32 {
        self.centerx + self.width / 2.
    }

    pub(crate) fn top(&self) -> f32 {
        self.centery + self.height / 2.
    }

    pub(crate) fn bottom(&self) -> f32 {
        self.centery - self.height / 2.
    }
}