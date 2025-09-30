use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use rand::prelude::random;

use crate::individual::{
    Demog, Sex, spawn_individual
};
use crate::partner::Partner;
use crate::config::SimulationParameters;

pub struct GestationPlugin;

impl Plugin for GestationPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- GESTATION
        .add_systems(Update, (
            immaculate_conception,
            (
                conception,
                update_gestation,
            ).run_if(on_timer(Duration::from_secs_f32(CONCEPTION_TIMESTEP))),
        ));
    }
}

//-- GESTATION
const CONCEPTION_TIMESTEP: f32 = 1.0/52.0;
// MIN_CONCEPTION_AGE, MAX_CONCEPTION_AGE, CONCEPTION_RATE, GESTATION_DURATION now come from SimulationParameters

// ------ GESTATION ------

#[derive(Component)]
pub struct RemainingGestation(pub f32);

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
    query: Query<(Entity, &Demog, &Partner), Without<RemainingGestation>>,
    params: Res<SimulationParameters>
) {
    for (e, demog, partner) in query.iter() {
        if demog.sex == Sex::Female {
            if demog.age > params.min_conception_age && demog.age < params.max_conception_age {
                let conception_prob = 1.0 - (-CONCEPTION_TIMESTEP * params.conception_rate).exp(); // f32.exp() is e^(f32)
                if random::<f32>() < conception_prob {
                    eprintln!("{:?} conceived at age {} with partner {:?}!", e, demog.age, partner.0);
                    commands.entity(e).insert(RemainingGestation(params.gestation_duration));
                }
            }
        }
    }
}