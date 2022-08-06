use bevy::{prelude::{Color, Vec4}, ui::UiColor};
use bevy_tweening::Lens;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UiColorLens {
    pub start: Color,
    pub end: Color
}

impl Lens<UiColor> for UiColorLens {
    fn lerp(&mut self, target: &mut UiColor, ratio: f32) {
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        
        target.0 = value.into();
    }
}