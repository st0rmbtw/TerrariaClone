#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::DebugLines;
#[cfg(feature = "debug")]
use bevy::prelude::Vec3;
#[cfg(feature = "debug")]
use bevy::prelude::Color;

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
        self.left < other.right &&
            self.top() >= other.bottom() &&
            self.right > other.left &&
            self.bottom() <= other.top()
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

    #[cfg(feature = "debug")]
    pub(crate) fn draw_left_side(&self, debug_lines: &mut DebugLines, duration: f32, color: Color) {
        debug_lines.line_colored(
            Vec3::new(self.left, self.bottom(), 10.),
            Vec3::new(self.left, self.top(), 10.),
            duration,
            color
        );
    }

    #[cfg(feature = "debug")]
    pub(crate) fn draw_right_side(&self, debug_lines: &mut DebugLines, duration: f32, color: Color) {
        debug_lines.line_colored(
            Vec3::new(self.right, self.bottom(), 10.),
            Vec3::new(self.right, self.top(), 10.),
            duration,
            color
        );
    }

    #[cfg(feature = "debug")]
    pub(crate) fn draw_top_side(&self, debug_lines: &mut DebugLines, duration: f32, color: Color) {
        debug_lines.line_colored(
            Vec3::new(self.left, self.top(), 10.),
            Vec3::new(self.right, self.top(), 10.),
            duration,
            color
        );
    }
    #[cfg(feature = "debug")]
    pub(crate) fn draw_bottom_side(&self, debug_lines: &mut DebugLines, duration: f32, color: Color) {
        debug_lines.line_colored(
            Vec3::new(self.left, self.bottom(), 10.),
            Vec3::new(self.right, self.bottom(), 10.),
            duration,
            color
        );
    }
}