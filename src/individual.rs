use crate::window::{Size, Position};

use bevy::prelude::*;
use bevy::core::FixedTimestep;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::cmp::min;

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
        .add_system(add_new_partner_seekers)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SEEKING_TIMESTEP.into()))
                .with_system(match_partners)
                .with_system(position_near_partner)
        )
        .add_system(resolve_matches);
    }
}

const AGING_TIMESTEP: f32 = 1.0/12.0;
const SEEKING_TIMESTEP: f32 = 2.0;  // N.B. slower for testing via printout + visualization

const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);
const MAX_SPRITE_SIZE: f32 = 0.3;

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

pub struct BecomeAdultEvent(Entity, Sex);  // TODO: learn more about borrow lifetime to use ref to &Demog in Event arguments

#[derive(Component)]
pub struct PartnerSeeking;

#[derive(Component)]
pub struct Partner {
    e: Entity
}

#[derive(Default)]
pub struct AvailableSeekers {
    females: Vec<Entity>,
    males: Vec<Entity>
}
impl AvailableSeekers {
    fn add_seeker(&mut self, e: Entity, sex: Sex) {
        match sex {
            Sex::Female => self.females.push(e),
            _ => self.males.push(e),
        }
    }
}

#[derive(Component)]
pub struct Relationship;

#[derive(Component)]
pub struct Partners {
    e1: Entity,
    e2: Entity
}

// #[derive(Component)]
// struct Gestation {
//     remaining: f32,
// }

pub fn age_older(
    // time: Res<Time>, 
    mut query: Query<(&mut Demog, Option<&mut Size>)>
) {
    for (mut demog, opt_size) in query.iter_mut() {
        // demog.age += time.delta_seconds();  // if not FixedTimestep
        demog.age += AGING_TIMESTEP;

        if let Some(mut size) = opt_size {
            if demog.age < PARTNER_SEEKING_AGE {
                size.resize(size_for_age(demog.age));
            }
        }
    }
}

pub fn start_partner_seeking(
    mut ev_adult: EventWriter<BecomeAdultEvent>,
    mut commands: Commands, 
    query: Query<(Entity, &Demog), (Without<PartnerSeeking>, Without<Partner>)>
) {
    for (e, demog) in query.iter() {

        if demog.age > PARTNER_SEEKING_AGE {
            eprintln!("Entity {:?} beginning partner-seeking", e);
            commands.entity(e).insert(PartnerSeeking);
            
            ev_adult.send(BecomeAdultEvent(e, demog.sex));
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

// pub fn add_new_partner_seekers(
//     mut cache: ResMut<AvailableSeekers>,
//     mut ev_adult: EventReader<BecomeAdultEvent>,
// ) {
//     for ev in ev_adult.iter() {
//         cache.add_seeker(ev.0, ev.1);
//     }
// }

// nicer alternative to above?
pub fn add_new_partner_seekers(
    mut cache: ResMut<AvailableSeekers>,
    query: Query<(Entity, &Demog), Added<PartnerSeeking>>
) {
    for (e, d) in query.iter() {
        cache.add_seeker(e, d.sex);
    }
}

pub fn match_partners(
    mut cache: ResMut<AvailableSeekers>,
    mut commands: Commands,
) {
    // Dummy example matching scheme: FIFO
    
    let it1 = cache.females.iter(); // drain() removes from vector as iterated (with unequal length kept after zip()
    let it2 = cache.males.iter();  // ugh: can't borrow mutable cache more than once at a time (w/o extra Rust skills)

    for (e1, e2) in it1.zip(it2) {
        commands
            .spawn()
            .insert(Relationship)
            .insert(Partners{
                e1: *e1,
                e2: *e2
            });
        eprintln!("New relationship between {:?} and {:?}", e1, e2);
    }

    let min_len = min(cache.females.len(), cache.males.len());
    // eprintln!("Minimum queue of partner seekers = {}", min_len);

    cache.males = cache.males.split_off(min_len);  // leave unmatched partner-seekers for next round
    cache.females = cache.females.split_off(min_len);
}

pub fn resolve_matches(
    mut commands: Commands,
    query: Query<(&Relationship, &Partners), Added<Partners>>,
) {
    for (_, p) in query.iter() {
        commands.entity(p.e1).insert(Partner {e: p.e2}).remove::<PartnerSeeking>();
        commands.entity(p.e2).insert(Partner {e: p.e1}).remove::<PartnerSeeking>();
    }
}

pub fn position_near_partner(
    query: Query<(&Position, &Partner)>
) {
    for (pos, partner) in query.iter() {
        let partner_pos = query.get(partner.e).unwrap().0;
        let distance = pos.distance(partner_pos);
        eprintln!("Partner distance {}", distance);
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

fn size_for_age(age: f32) -> f32 {
    return MAX_SPRITE_SIZE * age / PARTNER_SEEKING_AGE;
}

fn add_individual(mut commands: Commands) {

    let sex: Sex = rand::random();
    let age = 10.0;
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
        .insert(Size::square(size_for_age(age)))
        .id();
    eprintln!("...in entity {:?}", individual_id);
}