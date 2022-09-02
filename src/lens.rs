use bevy::{prelude::{Color, Vec4}, ui::UiColor, text::{Text}};
use interpolation::Lerp;

use crate::animation::Lens;

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextFontSizeLens {
    pub start: f32,
    pub end: f32
}

impl Lens<Text> for TextFontSizeLens {
    fn lerp(&mut self, target: &mut Text, ratio: f32) {
        target.sections[0].style.font_size = self.start.lerp(&self.end, &ratio);
    }
}