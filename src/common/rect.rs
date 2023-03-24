#[derive(Clone, PartialEq, Debug, Default)]
pub(crate) struct FRect {
    pub(crate) centerx: f32,
    pub(crate) centery: f32,
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) width: f32,
    pub(crate) height: f32
}

impl FRect {
    pub(crate) fn new_center(centerx: f32, centery: f32, width: f32, height: f32) -> Self {
        Self { 
            left: centerx - width / 2.,
            right: centerx + width / 2.,
            centerx,
            centery,
            width,
            height
        }
    }

    pub(crate) fn new_bounds_h(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            centerx: left + width / 2.,
            centery: top - height / 2.,
            right: left + width,
            left,
            width,
            height
        }
    }

    pub(crate) fn intersects(&self, other: &FRect) -> bool {
        return self.left < other.right &&
               self.top() >= other.bottom() &&
               self.right > other.left &&
               self.bottom() <= other.top();
    }

    pub(crate) fn inside(&self, point: (f32, f32)) -> bool {
        point.0 > self.left && point.0 < self.right && point.1 > self.bottom() && point.1 < self.top()
    }

    pub(crate) fn top(&self) -> f32 {
        self.centery + self.height / 2.
    }

    pub(crate) fn bottom(&self) -> f32 {
        self.centery - self.height / 2.
    }
}