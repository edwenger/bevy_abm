use bevy::prelude::*;
use bevy::core::FixedTimestep;

use rand::prelude::random;

use crate::individual::{
    Demog, Sex, spawn_individual
};
use crate::partner::Partner;

pub struct GestationPlugin;

impl Plugin for GestationPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- GESTATION
        .add_system(immaculate_conception)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(CONCEPTION_TIMESTEP.into()))
                .with_system(conception)
                .with_system(update_gestation)
        );
    }
}

//-- GESTATION
const CONCEPTION_TIMESTEP: f32 = 1.0/52.0;  
const MIN_CONCEPTION_AGE: f32 = 25.0;
const MAX_CONCEPTION_AGE: f32 = 35.0;
const CONCEPTION_RATE: f32 = 0.5;
const GESTATION_DURATION: f32 = 40.0 / 52.0;

// ------ GESTATION ------

#[derive(Component)]
pub struct RemainingGestation(f32);

#[derive(Component)]
pub struct Mother(pub Entity);

pub fn update_gestation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RemainingGestation, &Demog)>
) {
    for (e, mut gestation, demog) in query.iter_mut() {
        gestation.0 -= CONCEPTION_TIMESTEP;

        if gestation.0 < 0.0 {
            commands.entity(e).remove::<RemainingGestation>();
            eprintln!("{:?} had a baby at age {}!", e, demog.age);

            spawn_individual(
                &mut commands,
                0.0,    // age = newborn
                Some(e) // mother's entity_id
            );
        }
    }
}

pub fn immaculate_conception() {
    // TODO: placeholder for fecundity-rate dependent insertion of RemainingGestation w/o relationship
}

pub fn conception(
    mut commands: Commands,
    query: Query<(Entity, &Demog, &Partner), Without<RemainingGestation>>
) {
    for (e, demog, _partner) in query.iter() {
        if demog.sex == Sex::Female {
            if demog.age > MIN_CONCEPTION_AGE && demog.age < MAX_CONCEPTION_AGE {
                let conception_prob = 1.0 - (-CONCEPTION_TIMESTEP * CONCEPTION_RATE).exp(); // f32.exp() is e^(f32)
                if random::<f32>() < conception_prob {
                    eprintln!("{:?} conceived at age {}!", e, demog.age);
                    commands.entity(e).insert(RemainingGestation(GESTATION_DURATION));
                }
            }
        }
    }
}