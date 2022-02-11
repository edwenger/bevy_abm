use crate::window::{Size, Position};

use bevy::prelude::*;
use bevy::core::FixedTimestep;

pub struct HouseholdPlugin;

impl Plugin for HouseholdPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(household_spawner),
        );
    }
}

const HOUSEHOLD_COLOR: Color = Color::rgb(0.1, 0.4, 0.2);

#[derive(Component)]
struct Household;

fn household_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: HOUSEHOLD_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Household)
        .insert(Position::random_cell())
        .insert(Size::square(0.8));
}