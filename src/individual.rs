use crate::window::{Size, Position};

use bevy::prelude::*;
use bevy::core::FixedTimestep;

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_startup_system(add_individual)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(AGING_TIMESTEP.into()))
                .with_system(age_older),
        )
        .add_system(start_partner_seeking);
    }
}

const AGING_TIMESTEP: f32 = 1.0;

// const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
// const MALE_COLOR: Color = Color::rgb(0.1, 0.1, 0.4);
const FEMALE_COLOR: Color = Color::rgb(0.3, 0.1, 0.2);

const PARTNER_SEEKING_AGE: f32 = 15.0;

#[derive(Debug)]
pub enum Sex {
    Male,
    Female,
}

impl Default for Sex {
    fn default() -> Self { Sex::Female }
}

#[derive(Component)]
pub struct Individual;

#[derive(Component, Default, Debug)]
pub struct Demog {
    pub age: f32,
    pub sex: Sex,
}

// #[derive(Component)]
// struct Gestation {
//     remaining: f32,
// }

#[derive(Component)]
pub struct PartnerSeeking;

pub fn age_older(
    // time: Res<Time>, 
    mut query: Query<(Entity, &mut Demog)>) {
    for (e, mut demog) in query.iter_mut() {
        eprintln!("Entity {:?} is {}-year-old {:?}", e, demog.age, demog.sex);

        // demog.age += time.delta_seconds();  // if not FixedTimestep
        demog.age += AGING_TIMESTEP;
    }
}

pub fn start_partner_seeking(mut commands: Commands, query: Query<(Entity, &Demog, Without<PartnerSeeking>)>) {
    for (e, demog, _) in query.iter() {

        if demog.age > PARTNER_SEEKING_AGE {
            eprintln!("Entity {:?} beginning partner-seeking", e);
            commands.entity(e).insert(PartnerSeeking);
        }
    }
}

fn add_individual(mut commands: Commands) {
    let individual_id = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FEMALE_COLOR,  // TODO: link color/shape to age/sex
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Individual)
        .insert(Demog{
            age: 20.0,
            ..Default::default()  // sex: Sex::Female
        })
        .insert(Position::random_cell())
        .insert(Size::square(0.2))  // TODO: link size to age
        .id();
    eprintln!("First individual {:?}", individual_id);
}