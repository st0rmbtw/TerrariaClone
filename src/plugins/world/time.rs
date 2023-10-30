use std::{time::Duration, fmt::{Display, Write}};

use bevy::{prelude::{Plugin, App, Resource, IntoSystemConfigs, ResMut, Vec3, FixedUpdate}, time::common_conditions::on_fixed_timer};

use crate::plugins::InGameSystemSet;

pub(super) struct WorldTimePlugin;
impl Plugin for WorldTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            update_time
                .in_set(InGameSystemSet::FixedUpdate)
                .run_if(on_fixed_timer(Duration::from_millis(1)))
        );
    }
}

#[derive(Clone, Copy, Resource)]
pub(crate) struct GameTime {
    pub(crate) value: u32,
    is_day: bool,
    pub(crate) paused: bool,
}

impl GameTime {
    pub(crate) const MAX_TIME: u32 = 24 * 60 * 60;
    const DAY_DURATION: u32 = 15 * 60 * 60;
    const NIGHT_DURATION: u32 = Self::MAX_TIME - Self::DAY_DURATION;
    const MIDNIGHT: u32 = Self::NIGHT_DURATION / 2;
    const NIGHT_START: u32 = (19 * 60 * 60) + 30 * 60;
    const DAY_START: u32 = (4 * 60 * 60) + 30 * 60;

    pub(crate) const fn new(time: u32, is_day: bool, paused: bool) -> Self {
        Self {
            value: time,
            is_day,
            paused,
        }
    }

    pub(crate) const fn from_time(time: u32, paused: bool) -> Self {
        let t = time % Self::MAX_TIME;
        if t >= Self::DAY_DURATION {
            Self::new(t - Self::DAY_DURATION, false, paused)
        } else {
            Self::new(t, true, paused)
        }
    }

    fn tick(&mut self) {
        self.value += 50;
        if self.is_day {
            if self.value >= Self::DAY_DURATION {
                self.value = 0;
                self.is_day = false;
            }
        } else {
            if self.value >= Self::NIGHT_DURATION {
                self.value = 0;
                self.is_day = true;
            }
        }
    }

    pub(crate) fn get_ambient_color(&self) -> Vec3 {
        let mut color = Vec3::splat(255.);

        if self.is_day {
            if self.value < 13500 {
                let a = self.value as f32 / 13500.;
                color = Vec3::new(230., 220., 220.) * a + Vec3::new(25., 35., 35.);
            } else if self.value > 45900 {
                let a = 1.0 - (self.value as f32 / Self::DAY_DURATION as f32 - 0.85) * (10. / 1.5);
                color = Vec3::new(200., 85., 135.) * a + Vec3::splat(35.);
            } else if self.value > 37800 {
                let a = 1.0 - (self.value as f32 / Self::DAY_DURATION as f32 - 0.7) * (10. / 1.5);
                color = Vec3::new(20., 135., 85.) * a + Vec3::new(235., 120., 170.);
            }
        } else {
            if self.value < Self::MIDNIGHT {
                let a = 1. - (self.value as f32 / Self::MIDNIGHT as f32);
                color = Vec3::splat(30.) * a + Vec3::splat(5.);
            } else {
                let a = (self.value as f32 / Self::NIGHT_DURATION as f32 - 0.5) * 2.0;
                color = Vec3::new(20., 30., 30.) * a + Vec3::splat(5.);
            }
        }

        color / 255.
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::from_time(13500, false)
    }
}

impl Display for GameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = if self.is_day {
            Self::DAY_START + self.value
        } else {
            Self::NIGHT_START + self.value
        } % Self::MAX_TIME;

        let hours = time / 60 / 60;
        let minutes = time / 60 % 60;

        if hours < 10 {
            f.write_char('0')?;
        }
        write!(f, "{}", hours)?;
        f.write_char(':')?;
        if minutes < 10 {
            f.write_char('0')?;
        }
        write!(f, "{}", minutes)?;

        Ok(())
    }
}

fn update_time(mut game_time: ResMut<GameTime>) {
    if !game_time.paused {
        game_time.tick();
    }
}