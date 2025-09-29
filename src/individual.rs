use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::gestation::Mother;
use crate::config::SimulationParameters;

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- DEMOGRAPHICS
        .add_systems(Startup, initial_population)
        .add_systems(Update, (spawn_births, update_age.run_if(on_timer(Duration::from_secs_f32(AGING_TIMESTEP)))));
    }
}

//-- DEMOGRAPHICS
const AGING_TIMESTEP: f32 = 1.0/12.0;
// DEATH_AGE and PARTNER_SEEKING_AGE now come from SimulationParameters

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Sex {
    Female,
    Male,
}

impl Default for Sex {
    fn default() -> Self { Sex::Female }
}

impl Distribution<Sex> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Sex {
        match rng.gen_range(0..2) {
            0 => Sex::Female,
            _ => Sex::Male,
        }
    }
}

// ------ DEMOGRAPHICS ------

#[derive(Component)]
pub struct Individual;

#[derive(Component)]
pub struct Adult;

#[derive(Component, Default, Debug)]
pub struct Demog {
    pub age: f32,
    pub sex: Sex,
}

pub fn initial_population(mut commands: Commands) {
    /*
    startup_system to spawn initial individuals
        currently just a dummy function to test features="headless"
        TODO: add some Local<Configuration> to make it more useful
    */
    spawn_individual(&mut commands, 0.0, None);
}

pub fn spawn_individual(
    commands: &mut Commands,
    age: f32,
    mother_opt: Option<Entity>
) -> Entity {

    let sex: Sex = rand::random();

    eprintln!("Adding {}-year-old {:?}...", age, sex);
    let individual_id = commands
        .spawn((Individual, Demog{
            age: age,
            sex: sex,
        }))
        .id();

    if let Some(mother) = mother_opt  {
        commands.entity(individual_id).insert(Mother(mother));
    }

    eprintln!("...in entity {:?}", individual_id);
    return individual_id;
}

pub fn spawn_births() {
    // TODO: placeholder for birth-rate dependent spawning of new individuals w/o parents
}

pub fn update_age(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Demog, Option<&Adult>)>,
    params: Res<SimulationParameters>
) {
    for (e, mut demog, adult_opt) in query.iter_mut() {

        demog.age += AGING_TIMESTEP;

        if demog.age > params.partner_seeking_age && adult_opt.is_none() {
            eprintln!("{:?} is now an adult", e);
            commands.entity(e).insert(Adult);
        }

        if demog.age > params.death_age {
            eprintln!("{:?} died", e);
            commands.entity(e).despawn();
        }
    }
}