use std::{marker::PhantomData, ops::{Deref, DerefMut, Not}};

use bevy::prelude::Resource;

use crate::common::{BoolValue, Toggle};

pub(crate) trait VisibilityMarker: Clone + Copy + Send + Sync + 'static {}

#[derive(Clone, Copy)]
pub(crate) struct Cursor;
impl VisibilityMarker for Cursor {}

#[derive(Clone, Copy)]
pub(crate) struct Ui;
impl VisibilityMarker for Ui {}

#[derive(Clone, Copy)]
pub(crate) struct SettingsMenu;
impl VisibilityMarker for SettingsMenu {}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Visible<T: VisibilityMarker> {
    visible: bool,
    _phantom: PhantomData<T>
}

impl<T: VisibilityMarker> Visible<T> {
    pub(crate) fn visible() -> Self {
        Visible {
            visible: true,
            _phantom: PhantomData
        }
    }

    pub(crate) fn hidden() -> Self {
        Visible {
            visible: false,
            _phantom: PhantomData
        }
    }
}

impl<T: VisibilityMarker> Deref for Visible<T> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.visible
    }
}

impl<T: VisibilityMarker> DerefMut for Visible<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.visible
    }
}

impl<T: VisibilityMarker> Toggle for Visible<T> {
    fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl<T: VisibilityMarker> BoolValue for Visible<T> {
    fn value(&self) -> bool {
        self.visible
    }
}

impl<T: VisibilityMarker> Not for Visible<T> {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.visible
    }
}