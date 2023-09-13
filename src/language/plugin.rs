use bevy::{prelude::{Plugin, App, Query, Without, Res, Commands, Entity, PostUpdate, Component, Changed}, text::Text, ecs::query::{ReadOnlyWorldQuery, Has}};

use super::{LanguageContent, LocalizedText};

pub(crate) struct LanguagePlugin;
impl Plugin for LanguagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                localize_text::<Without<Localized>>,
                localize_text::<Changed<LocalizedText>>,
            )
        );
    }
}

#[derive(Component)]
struct Localized;

fn localize_text<TFilter: ReadOnlyWorldQuery>(
    mut commands: Commands,
    language_content: Res<LanguageContent>,
    mut query: Query<(Entity, &mut Text, &LocalizedText, Has<Localized>), TFilter>
) {
    for (entity, mut text, localized_text, localized) in &mut query {
        text.sections[0].value = localized_text.format(&language_content);

        if !localized {
            commands.entity(entity).insert(Localized);
        }
    }
}