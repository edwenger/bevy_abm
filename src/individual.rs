use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::gestation::Mother;
use crate::config::{SimulationParameters, Args};

use serde::Serialize;

#[derive(Event, Serialize, Clone)]
pub struct BirthEvent {
    pub child_entity: Entity,
    pub mother_entity: Option<Entity>,
    pub time: f32,
}

#[derive(Event, Serialize, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
    pub age: f32,
    pub time: f32,
}

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- DEMOGRAPHICS
        .add_event::<BirthEvent>()
        .add_event::<DeathEvent>()
        .add_systems(Startup, initial_population)
        .add_systems(Update, (
            spawn_births,
            update_age.run_if(on_timer(Duration::from_secs_f32(AGING_TIMESTEP))),
            check_simulation_end
        ));
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

#[derive(Component)]
pub struct Elder;

#[derive(Component, Default, Debug)]
pub struct Demog {
    pub age: f32,
    pub sex: Sex,
}

pub fn initial_population(
    mut commands: Commands,
    args: Res<Args>,
    mut birth_events: EventWriter<BirthEvent>,
    time: Res<Time>
) {
    // Spawn initial population based on command line argument
    // In headless mode, spawn N individuals as specified
    // In GUI mode, spawn 0 by default (can be overridden with -n)
    for _ in 0..args.initial_population {
        // Spawn individuals with random ages between 18-30 for variety
        let age = 18.0 + rand::random::<f32>() * 12.0; // 18-30 years old
        spawn_individual(&mut commands, age, None, &mut birth_events, &time);
    }

    if args.initial_population > 0 {
        info!("Spawned {} initial individuals", args.initial_population);
    }
}

pub fn spawn_individual(
    commands: &mut Commands,
    age: f32,
    mother_opt: Option<Entity>,
    birth_events: &mut EventWriter<BirthEvent>,
    time: &Res<Time>
) -> Entity {

    let sex: Sex = rand::random();

    debug!("Adding {}-year-old {:?} in entity (pending)", age, sex);
    let individual_id = commands
        .spawn((Individual, Demog{
            age: age,
            sex: sex,
        }))
        .id();

    if let Some(mother) = mother_opt  {
        commands.entity(individual_id).insert(Mother(mother));
    }

    // Emit birth event
    birth_events.send(BirthEvent {
        child_entity: individual_id,
        mother_entity: mother_opt,
        time: time.elapsed_seconds(),
    });

    debug!("Spawned individual {:?}", individual_id);
    return individual_id;
}

pub fn spawn_births() {
    // TODO: placeholder for birth-rate dependent spawning of new individuals w/o parents
}

pub fn update_age(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Demog, Option<&Adult>, Option<&Elder>)>,
    params: Res<SimulationParameters>,
    mut death_events: EventWriter<DeathEvent>,
    time: Res<Time>
) {
    for (e, mut demog, adult_opt, elder_opt) in query.iter_mut() {

        demog.age += AGING_TIMESTEP;

        if demog.age > params.min_partner_seeking_age && adult_opt.is_none() {
            debug!("{:?} is now an adult", e);
            commands.entity(e).insert(Adult);
        }

        if demog.age > params.max_partner_seeking_age && elder_opt.is_none() {
            debug!("{:?} is now an elder", e);
            commands.entity(e).insert(Elder);
        }

        if demog.age > params.death_age {
            debug!("{:?} died", e);

            death_events.send(DeathEvent {
                entity: e,
                age: demog.age,
                time: time.elapsed_seconds(),
            });

            commands.entity(e).despawn();
        }
    }
}

pub fn check_simulation_end(
    args: Res<Args>,
    time: Res<Time>,
    mut exit: EventWriter<AppExit>
) {
    if let Some(sim_years) = args.sim_years {
        let elapsed_years = time.elapsed_seconds();

        if elapsed_years >= sim_years {
            info!("Simulation completed after {:.2} years", elapsed_years);
            info!("Exiting...");
            exit.send(AppExit);
        }
    }
}