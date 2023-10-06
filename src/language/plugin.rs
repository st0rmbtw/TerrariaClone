use bevy::{prelude::{Plugin, App, Query, Without, Res, Commands, Entity, PostUpdate, Component, Changed, Or}, text::Text, ecs::query::Has};

use super::{LanguageContent, LocalizedText};

pub(crate) struct LanguagePlugin;
impl Plugin for LanguagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, localize_text);
    }
}

#[derive(Component)]
struct Localized;

fn localize_text(
    mut commands: Commands,
    language_content: Res<LanguageContent>,
    mut query: Query<(Entity, &mut Text, &mut LocalizedText, Has<Localized>), Or<(Without<Localized>, Changed<LocalizedText>)>>
) {
    for (entity, mut text, mut localized_text, localized) in &mut query {
        text.sections[0].value = localized_text.text(&language_content);

        if !localized {
            commands.entity(entity).insert(Localized);
        }
    }
}