use bevy::{prelude::*, sprite::Anchor};

/// Layer speed type.
/// Layers with horizontal or vertical speed are only able to travel in one direction,
/// while bidirectional layers can be scrolled endlessly in both directions.
#[derive(Debug)]
pub enum LayerSpeed {
    Horizontal(f32),
    Vertical(f32),
    Bidirectional(f32, f32),
}

/// Layer initialization data
#[derive(Debug)]
pub struct LayerData {
    /// Relative speed of layer to the camera movement
    pub speed: LayerSpeed,

    pub image: Handle<Image>,
    /// Scale of the texture
    pub scale: f32,
    /// Z position of the layer
    pub z: f32,
    /// Default initial position of the Entity container
    pub position: Vec2,
    /// Number used to determine when textures are moved to opposite side of camera
    pub transition_factor: f32,

    pub anchor: Anchor,

    pub fill_screen_height: bool
}

impl Default for LayerData {
    fn default() -> Self {
        Self {
            speed: LayerSpeed::Horizontal(1.),
            image: Handle::default(),
            scale: 1.0,
            z: 0.0,
            position: Vec2::ZERO,
            transition_factor: 1.2,
            fill_screen_height: false,
            anchor: Anchor::default()
        }
    }
}

/// Core component for parallax layer
#[derive(Component)]
pub struct LayerComponent {
    /// Relative speed of layer to the camera movement
    pub speed: Vec2,
    /// Number of textures in the layer
    pub texture_count: f32,
    /// Number used to determine when textures are moved to opposite side of camera
    pub transition_factor: f32,
}

/// Core component for layer texture
#[derive(Component)]
pub struct LayerTextureComponent {
    /// Width of the texture
    pub width: f32,
}
