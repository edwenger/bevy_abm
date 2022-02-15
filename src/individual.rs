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
        .add_event::<BecomeAdultEvent>()
        .add_startup_system(add_individual)
        .add_system(keyboard_input)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(AGING_TIMESTEP.into()))
                .with_system(age_older),
        )
        .add_system(start_partner_seeking)
        .add_system(recolor_new_adults)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SEEKING_TIMESTEP.into()))
                .with_system(seek_partner),
        );
    }
}

const AGING_TIMESTEP: f32 = 1.0/52.0;
const SEEKING_TIMESTEP: f32 = 1.0/4.0;

const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);

const PARTNER_SEEKING_AGE: f32 = 20.0;

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
        match rng.gen_range(0, 2) {
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

pub struct BecomeAdultEvent(Entity, Sex);

#[derive(Component)]
pub struct PartnerSeeking;

#[derive(Default)]
pub struct AvailableSeekers {
    females: Vec<Entity>,
    males: Vec<Entity>
}

// #[derive(Component)]
// struct Gestation {
//     remaining: f32,
// }

pub fn age_older(
    // time: Res<Time>, 
    mut query: Query<&mut Demog>
) {
    for mut demog in query.iter_mut() {
        // demog.age += time.delta_seconds();  // if not FixedTimestep
        demog.age += AGING_TIMESTEP;
    }
}

pub fn start_partner_seeking(
    mut cache: ResMut<AvailableSeekers>,
    mut ev_adult: EventWriter<BecomeAdultEvent>,
    mut commands: Commands, 
    query: Query<(Entity, &Demog, Without<PartnerSeeking>)>
) {
    for (e, demog, _) in query.iter() {

        if demog.age > PARTNER_SEEKING_AGE {
            eprintln!("Entity {:?} beginning partner-seeking", e);
            commands.entity(e).insert(PartnerSeeking);
            
            ev_adult.send(BecomeAdultEvent(e, demog.sex));

            match demog.sex {
                Sex::Female => cache.females.push(e),
                _ => cache.males.push(e),
            }
        }
    }
}

pub fn recolor_new_adults(
    mut commands: Commands,
    mut ev_adult: EventReader<BecomeAdultEvent>,
) {
    for ev in ev_adult.iter() {
        eprintln!("Processing new adult {:?}", ev.0);
        commands.entity(ev.0)
            .remove_bundle::<SpriteBundle>()
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: color_for_sex(ev.1),
                    ..Default::default()
                },
                ..Default::default()
            });
    }
}

// pub fn match_partners() {
    
// }

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

fn color_for_sex(sex: Sex) -> Color {
    return if sex==Sex::Female {
        FEMALE_COLOR
    } else {
        MALE_COLOR
    }
}

fn add_individual(mut commands: Commands) {

    let sex: Sex = rand::random();
    let age = 15.0;
    let color = if age < PARTNER_SEEKING_AGE {
        CHILD_COLOR
    } else {
        color_for_sex(sex)
    };

    eprintln!("Adding {}-year-old {:?}...", age, sex);
    let individual_id = commands
        .spawn()
        .insert(Individual)
        .insert(Demog{
            age: age,
            sex: sex,
        })
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: color,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Position::random_cell())
        .insert(Size::square(0.3))  // TODO: link size to age
        .id();
    eprintln!("...in entity {:?}", individual_id);
}