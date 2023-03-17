use std::ops::Mul;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct FRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct URect {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct IRect {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl URect {
    pub fn to_frect(&self) -> FRect {
        FRect {
            left: self.left as f32,
            right: self.right as f32,
            top: self.top as f32,
            bottom: self.bottom as f32,
        }
    }
}

impl FRect {
    pub fn intersect(&self, rect: FRect) -> bool {
        self.left < rect.right
            && self.right > rect.left
            && self.bottom > rect.top
            && self.top > rect.bottom
    }

    
    pub fn inside(&self, point: (f32, f32)) -> bool {
        point.0 > self.left && point.0 < self.right && point.1 > self.bottom && point.1 < self.top
    }
}

impl Mul<f32> for FRect {
    type Output = FRect;

    fn mul(self, rhs: f32) -> Self::Output {
        FRect {
            left: self.left * rhs,
            right: self.right * rhs,
            top: self.top * rhs,
            bottom: self.bottom * rhs,
        }
    }
}