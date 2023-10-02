use bevy::{prelude::{Plugin, App, World, default, Vec2, Transform, Update, IntoSystemConfigs, FixedUpdate, Commands, apply_deferred, Assets, Image}, ecs::system::Command, sprite::SpriteBundle, ui::Interaction, time::Timer};

use crate::{items::ItemStack, common::{components::{EntityRect, Velocity}, rect::FRect}, language::{LocalizedText, keys::ItemStringKey, args}};

use self::components::{DroppedItem, GrabTimer};

use super::{assets::ItemAssets, world::{constants::TILE_SIZE, WORLD_RENDER_LAYER}, InGameSystemSet, cursor::components::Hoverable};

mod systems;
mod components;
pub(super) struct ItemPlugin;
impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    systems::stack_items,
                    apply_deferred,
                    systems::follow_player,
                )
                .chain(),

                (
                    systems::gravity,
                    systems::air_resistance,
                ),
                systems::detect_collisions,
                systems::update_item_rect,
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            Update,
            (
                systems::rotate_item,
                systems::move_item,
                systems::update_item_hoverable_info
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

const STACK_RANGE: f32 = 1.5 * TILE_SIZE;
const GRAVITY: f32 = 0.1;
const MAX_VERTICAL_SPEED: f32 = 5.;
const MAX_HORIZONTAL_SPEED: f32 = 5.;
const GRAB_RANGE: f32 = 5.25 * TILE_SIZE;


struct SpawnDroppedItemCommand {
    position: Vec2,
    velocity: Vec2,
    item_stack: ItemStack,
    grab_timer: Option<Timer>
}

impl Command for SpawnDroppedItemCommand {
    fn apply(self, world: &mut World) {
        let item_assets = world.resource::<ItemAssets>();
        let images = world.resource::<Assets<Image>>();

        let texture = item_assets.get_by_item(self.item_stack.item);
        let image = images.get(&texture).unwrap();

        let size = image.size();

        let mut entity = world.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(self.position.x, self.position.y, 10.),
                ..default()
            },
            EntityRect(FRect::new_center(self.position.x, self.position.y, size.x, size.y)),
            Velocity(self.velocity),
            Hoverable::SimpleText(item_hoverable_text(self.item_stack)),
            Interaction::default(),
            DroppedItem {
                item_stack: self.item_stack
            },
            WORLD_RENDER_LAYER
        ));

        if let Some(timer) = self.grab_timer {
            entity.insert(GrabTimer(timer));
        }
    }
}

pub(crate) trait ItemCommandsExt {
    fn spawn_dropped_item(&mut self, position: Vec2, velocity: Vec2, item_stack: ItemStack, timer: Option<Timer>);
}

impl ItemCommandsExt for Commands<'_, '_> {
    fn spawn_dropped_item(&mut self, position: Vec2, velocity: Vec2, item_stack: ItemStack, timer: Option<Timer>) {
        self.add(SpawnDroppedItemCommand {
            position,
            velocity,
            item_stack,
            grab_timer: timer
        });
    }
}

#[inline]
fn item_hoverable_text(item_stack: ItemStack) -> LocalizedText {
    LocalizedText::new(
        ItemStringKey::get_by_item(item_stack.item),
        if item_stack.stack > 1 { "{} ({})" } else { "{}" },
        if item_stack.stack > 1 { args![item_stack.stack] } else { args![] },
    )
}