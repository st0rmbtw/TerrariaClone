use bevy::{ecs::system::EntityCommands, prelude::Component};

pub trait EntityCommandsExtensions<'w, 's, 'a> {
    fn insert_if(
        &mut self,
        component: impl Component,
        insert: bool,
    ) -> &mut EntityCommands<'w, 's, 'a>;
}

impl<'w, 's, 'a> EntityCommandsExtensions<'w, 's, 'a> for EntityCommands<'w, 's, 'a> {
    fn insert_if(
        &mut self,
        component: impl Component,
        insert: bool,
    ) -> &mut EntityCommands<'w, 's, 'a> {
        if insert {
            self.insert(component);
        }

        self
    }
}