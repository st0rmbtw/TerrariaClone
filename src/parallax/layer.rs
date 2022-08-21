use bevy::prelude::*;

/// Layer initialization data
#[derive(Debug)]
pub struct LayerData {
    /// Relative speed of layer to the camera movement
    pub speed: f32,
    /// Path to layer texture file
    pub path: String,
    
    pub image: Handle<TextureAtlas>,
    /// Scale of the texture
    pub scale: f32,
    /// Z position of the layer
    pub z: f32,
    /// Default initial position of the Entity container
    pub position: Vec2,
    /// Number used to determine when textures are moved to opposite side of camera
    pub transition_factor: f32,
}

impl Default for LayerData {
    fn default() -> Self {
        Self {
            speed: 1.0,
            path: "".to_string(),
            image: Handle::default(),
            scale: 1.0,
            z: 0.0,
            position: Vec2::ZERO,
            transition_factor: 1.2,
        }
    }
}

/// Core component for parallax layer
#[derive(Component)]
pub struct LayerComponent {
    /// Relative speed of layer to the camera movement
    pub speed: f32,
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