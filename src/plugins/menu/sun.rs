use std::time::Duration;

use autodefault::autodefault;
use bevy::{prelude::{Commands, ImageBundle, Res, Component}, ui::{UiImage, UiRect, Val, PositionType, Style}};

use crate::{plugins::assets::BackgroundAssets, animation::{Tween, EaseMethod, lens::UiPositionLens, Animator, RepeatStrategy, Tracks, RepeatCount}};

#[derive(Component)]
pub struct Sun;

#[autodefault(except(UiPositionLens))]
pub(super) fn setup_sun(
    mut commands: Commands,
    background_assets: Res<BackgroundAssets>
) {

    let x_animation = Tween::new(
        EaseMethod::Linear,
        RepeatStrategy::MirroredRepeat,
        Duration::from_secs(25),
        UiPositionLens {
            start: UiRect {
                left: Val::Percent(0.)
            },
            end: UiRect {
                left: Val::Percent(100.)
            }
        }
    )
    .with_repeat_count(RepeatCount::Infinite);

    let y_animation = Tween::new(
        EaseMethod::CustomFunction(|x| {
            if x > 0.5 {
                1. - x
            } else {
                x
            }
        }),
        RepeatStrategy::MirroredRepeat,
        Duration::from_secs(25),
        UiPositionLens {
            start: UiRect {
                bottom: Val::Percent(50.),
            },
            end: UiRect {
                bottom: Val::Percent(100.),
            }
        }
    )
    .with_repeat_count(RepeatCount::Infinite);

    let logo_animation = Tracks::new([
        y_animation,
        x_animation,
    ]);

    commands.spawn((
        ImageBundle {
            style: Style {
                position_type: PositionType::Absolute
            },
            image: UiImage(background_assets.sun.clone())
        },
        Animator::new(logo_animation),
        Sun
    ));
}