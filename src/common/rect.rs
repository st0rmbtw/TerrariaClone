use bevy::prelude::Vec2;

#[cfg(feature = "debug")]
use bevy::prelude::{Gizmos, Color, Reflect};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct FRect {
    pub(crate) centerx: f32,
    pub(crate) centery: f32,
    width: f32,
    height: f32
}

impl FRect {
    #[inline]
    pub(crate) fn new_center(centerx: f32, centery: f32, width: f32, height: f32) -> Self {
        Self { 
            centerx,
            centery,
            width,
            height
        }
    }

    #[inline]
    pub(crate) fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            centerx: left + width / 2.,
            centery: top - height / 2.,
            width,
            height
        }
    }

    #[inline(always)]
    pub(crate) fn center(&self) -> Vec2 {
        Vec2::new(self.centerx, self.centery)
    }

    #[inline]
    pub(crate) fn intersects(&self, other: &FRect) -> bool {
        self.left() < other.right() &&
            self.top() > other.bottom() &&
            self.right() > other.left() &&
            self.bottom() < other.top()
    }

    #[inline]
    pub(crate) fn contains(&self, point: Vec2) -> bool {
        point.x > self.left() && point.x < self.right() && point.y > self.bottom() && point.y < self.top()
    }

    #[inline]
    pub(crate) fn top(&self) -> f32 {
        self.centery + self.height / 2.
    }

    #[inline]
    pub(crate) fn bottom(&self) -> f32 {
        self.centery - self.height / 2.
    }

    #[inline]
    pub(crate) fn left(&self) -> f32 {
        self.centerx - self.width / 2.
    }

    #[inline]
    pub(crate) fn right(&self) -> f32 {
        self.centerx + self.width / 2.
    }

    #[inline]
    pub(crate) fn height(&self) -> f32 {
        self.height
    }

    #[inline]
    pub(crate) fn width(&self) -> f32 {
        self.width
    }

    #[inline]
    pub(crate) fn half_height(&self) -> f32 {
        self.height / 2.
    }

    #[inline]
    pub(crate) fn half_width(&self) -> f32 {
        self.width / 2.
    }

    #[cfg(feature = "debug")]
    pub(crate) fn draw_left_side(&self, debug_lines: &mut Gizmos, color: Color) {
        debug_lines.line_2d(
            Vec2::new(self.left(), self.bottom()),
            Vec2::new(self.left(), self.top()),
            color
        );
    }

    #[cfg(feature = "debug")]
    pub(crate) fn draw_right_side(&self, debug_lines: &mut Gizmos, color: Color) {
        debug_lines.line_2d(
            Vec2::new(self.right(), self.bottom()),
            Vec2::new(self.right(), self.top()),
            color
        );
    }

    #[cfg(feature = "debug")]
    pub(crate) fn draw_top_side(&self, debug_lines: &mut Gizmos, color: Color) {
        debug_lines.line_2d(
            Vec2::new(self.left(), self.top()),
            Vec2::new(self.right(), self.top()),
            color
        );
    }
    #[cfg(feature = "debug")]
    pub(crate) fn draw_bottom_side(&self, debug_lines: &mut Gizmos, color: Color) {
        debug_lines.line_2d(
            Vec2::new(self.left(), self.bottom()),
            Vec2::new(self.right(), self.bottom()),
            color
        );
    }
}