use crate::window::{Size, Position};

use bevy::prelude::*;
use bevy::core::FixedTimestep;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<AvailableSeekers>()
        .add_startup_system(add_individual)
        .add_system(keyboard_input)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(AGING_TIMESTEP.into()))
                .with_system(age_older),
        )
        .add_system(start_partner_seeking)
        .add_system(seek_partner);
    }
}

const AGING_TIMESTEP: f32 = 1.0;

const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);

const PARTNER_SEEKING_AGE: f32 = 15.0;

#[derive(Debug, PartialEq)]
pub enum Sex {
    Female,
    Male,
}

impl Default for Sex {
    fn default() -> Self { Sex::Female }
}

impl Distribution<Sex> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Sex {
        match rng.gen_range(0, 2) { // rand 0.5, 0.6, 0.7
        // match rng.gen_range(0..=1) { // rand 0.8
            0 => Sex::Female,
            _ => Sex::Male,
        }
    }
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

#[derive(Default)]
pub struct AvailableSeekers {
    females: Vec<Entity>,
    males: Vec<Entity>
}

pub fn age_older(
    // time: Res<Time>, 
    mut query: Query<(Entity, &mut Demog)>
) {
    for (e, mut demog) in query.iter_mut() {
        eprintln!("Entity {:?} is {}-year-old {:?}", e, demog.age, demog.sex);

        // demog.age += time.delta_seconds();  // if not FixedTimestep
        demog.age += AGING_TIMESTEP;
    }
}

pub fn start_partner_seeking(
    mut cache: ResMut<AvailableSeekers>, 
    mut commands: Commands, 
    query: Query<(Entity, &Demog, Without<PartnerSeeking>)>
) {
    for (e, demog, _) in query.iter() {

        if demog.age > PARTNER_SEEKING_AGE {
            eprintln!("Entity {:?} beginning partner-seeking", e);
            commands.entity(e).insert(PartnerSeeking);
            
            match demog.sex {
                Sex::Female => cache.females.push(e),
                _ => cache.males.push(e),
            }
        }
    }
}

pub fn seek_partner(
    mut cache: ResMut<AvailableSeekers>, 
    // mut commands: Commands, 
    query: Query<(Entity, &Demog, With<PartnerSeeking>)>   
) {
    for (e, demog, _) in query.iter() {
        let candidates = match demog.sex {
            Sex::Female => &mut cache.males,
            _ => &mut cache.females
        };
        
        while let Some(candidate) = candidates.pop() {
            eprintln!("Candidate partner for {:?} is {:?}", e, candidate);
        }
    }
}

fn keyboard_input(
    commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        // Space was pressed --> add a random person
        add_individual(commands);
    }
}

fn add_individual(mut commands: Commands) {

    let sex: Sex = rand::random();
    let age = 20.0;
    let color = if age < PARTNER_SEEKING_AGE {
        CHILD_COLOR
    } else if sex==Sex::Female {
        FEMALE_COLOR
    } else {
        MALE_COLOR
    };

    let individual_id = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Individual)
        .insert(Demog{
            age: age,
            sex: sex,
        })
        .insert(Position::random_cell())
        .insert(Size::square(0.2))  // TODO: link size to age
        .id();
    eprintln!("First individual {:?}", individual_id);
}