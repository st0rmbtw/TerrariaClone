use bevy::{
    prelude::{Color, Vec4, Transform},
    text::Text,
    ui::BackgroundColor,
};
use interpolation::Lerp;

use crate::animation::Lens;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BackgroundColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<BackgroundColor> for BackgroundColorLens {
    fn lerp(&mut self, target: &mut BackgroundColor, ratio: f32) {
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);

        target.0 = value.into();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextFontSizeLens {
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformLens {
    pub start: Transform,
    pub end: Transform,
}

impl Lens<Text> for TextFontSizeLens {
    fn lerp(&mut self, target: &mut Text, ratio: f32) {
        target.sections[0].style.font_size = self.start.lerp(&self.end, &ratio);
    }
}

impl Lens<Transform> for TransformLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        target.translation = self.start.translation.lerp(self.end.translation, ratio);
        target.rotation = self.start.rotation.lerp(self.end.rotation, ratio);
        target.scale = self.start.scale.lerp(self.end.scale, ratio);
    }
}