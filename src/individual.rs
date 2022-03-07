use crate::window::{Size, Position};

use std::cmp::min;

use bevy::prelude::*;
use bevy::core::FixedTimestep;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rand::prelude::random;

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- DEMOGRAPHICS
        .add_startup_system(initial_population)
        .add_system(update_births)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(AGING_TIMESTEP.into()))
                .with_system(update_age)
        )

        //-- DISPLAY
        .add_system(recolor_new_adults)
        .add_system(move_towards)
        .add_system(update_size)

        //-- PARTNERS
        .init_resource::<AvailableSeekers>()
        .add_system(start_partner_seeking)
        .add_system(add_new_partner_seekers)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SEEKING_TIMESTEP.into()))
                .with_system(match_partners)
                .with_system(set_partner_destination)
        )
        .add_system(resolve_matches)

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


//-- DEMOGRAPHICS
const AGING_TIMESTEP: f32 = 1.0/12.0;
const DEATH_AGE: f32 = 60.0;  // TODO: if this is young enough it exposes runtime error when dead singles are still in FIFO partner matching queue

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

//-- DISPLAY
const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);
const MIN_SPRITE_SIZE: f32 = 0.05;
const MAX_SPRITE_SIZE: f32 = 0.3;
const MOVE_VELOCITY: f32 = 5.0;

//-- PARTNERS
const SEEKING_TIMESTEP: f32 = 1.0/4.0;  // N.B. slower for testing via printout + visualization
const PARTNER_SEEKING_AGE: f32 = 20.0;

//-- GESTATION
const CONCEPTION_TIMESTEP: f32 = 1.0/52.0;  
const MIN_CONCEPTION_AGE: f32 = 25.0;
const MAX_CONCEPTION_AGE: f32 = 35.0;
const CONCEPTION_RATE: f32 = 0.5;
const GESTATION_DURATION: f32 = 40.0 / 52.0;


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
    position: Option<Position>
) -> Entity {

    let sex: Sex = rand::random();
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
        .insert(Size::square(size_for_age(age)))
        .id();

    match position {
        Some(x) => commands.entity(individual_id).insert(x),
        None    => commands.entity(individual_id).insert(Position::random_cell())
    };

    eprintln!("...in entity {:?}", individual_id);
    return individual_id;
}

pub fn update_births() {
    // TODO: placeholder for birth-rate dependent spawning of new individuals w/o parents
}

pub fn update_age(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Demog, Option<&Adult>)>
) {
    for (e, mut demog, adult_opt) in query.iter_mut() {

        demog.age += AGING_TIMESTEP;

        if demog.age > PARTNER_SEEKING_AGE && adult_opt.is_none() {
            eprintln!("{:?} is now an adult", e);
            commands.entity(e).insert(Adult);
        }

        if demog.age > DEATH_AGE {
            eprintln!("{:?} died", e);
            commands.entity(e).despawn();
        }
    }
}

// ------ DISPLAY ------

#[derive(Component)]
pub struct MovingTowards {
    destination: Position
}

pub fn update_size(mut query: Query<(&Demog, &mut Size), Without<Adult>>) {
    for (demog, mut size) in query.iter_mut() {
        size.resize(size_for_age(demog.age));
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
    return MIN_SPRITE_SIZE + (MAX_SPRITE_SIZE - MIN_SPRITE_SIZE) * age / PARTNER_SEEKING_AGE;
}

pub fn recolor_new_adults(
    mut query: Query<(&Demog, &mut Sprite), Added<Adult>>,
) {
    for (demog, mut sprite) in query.iter_mut() {
        sprite.color = color_for_sex(demog.sex);
    }
}

pub fn set_partner_destination(
    mut commands: Commands,
    query: Query<(Entity, &Position, &Partner), Added<Partner>>
) {
    for (e, pos, partner) in query.iter() {
        let partner_pos = query.get(partner.e).unwrap().1;  // unwrap() takes non-error from Result<(Ent, Pos, Part), Error> type
        // eprintln! ("Position: {}, {}", pos.x, pos.y);
        // eprintln! ("Partner Position: {}, {}", partner_pos.x, partner_pos.y);
        let _distance = pos.distance(partner_pos);
        // eprintln!("Partner distance {}", _distance);
        let midpoint = pos.midpoint(partner_pos);
        // eprintln! ("Destination: {}, {}", midpoint.x, midpoint.y);
        commands.entity(e).insert(MovingTowards { destination: midpoint });
    }
}

pub fn move_towards(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &MovingTowards)>
) {
    for (e, mut pos, mov) in query.iter_mut() {
        let distance = pos.distance(&mov.destination);
        if distance > MAX_SPRITE_SIZE * 0.7071 {  // almost touching on diagonal
            let v = MOVE_VELOCITY * time.delta_seconds();
            let u = pos.unit_direction(&mov.destination);
            pos.x = pos.x + u.x * v;
            pos.y = pos.y + u.y * v;
        } else {
            commands.entity(e).remove::<MovingTowards>();
        }
    }
}

// ------ PARTNER ------

#[derive(Component)]
pub struct PartnerSeeking;

#[derive(Component)]
pub struct Partner {
    e: Entity
}

#[derive(Component)]
pub struct Relationship;

#[derive(Component)]
pub struct Partners {
    e1: Entity,
    e2: Entity
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

pub fn start_partner_seeking(
    mut commands: Commands, 
    query: Query<Entity, (Without<PartnerSeeking>, Without<Partner>, With<Adult>)>
) {
    for e in query.iter() {
        eprintln!("Entity {:?} beginning partner-seeking", e);
        commands.entity(e).insert(PartnerSeeking);            
    }
}

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

// ------ GESTATION ------

#[derive(Component)]
pub struct RemainingGestation(f32);

#[derive(Component)]
pub struct Mother(Entity);

pub fn update_gestation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RemainingGestation, &Demog, Option<&Position>)>
) {
    for (e, mut gestation, demog, opt_pos) in query.iter_mut() {
        gestation.0 -= CONCEPTION_TIMESTEP;

        if gestation.0 < 0.0 {
            commands.entity(e).remove::<RemainingGestation>();
            eprintln!("{:?} had a baby at age {}!", e, demog.age);

            // spawn the age=0 newborn child
            if let Some(pos) = opt_pos {
                spawn_individual(&mut commands, 0.0, Some(Position {
                    x: pos.x - 0.5 + random::<f32>(),
                    y: pos.y - 0.5 + random::<f32>()
                }));
            } else {
                spawn_individual(&mut commands, 0.0, None);
            }

            // add the child to the parent?? (but this messes with Transform + GlobalTransform position translation??)
            //commands.entity(e).push_children(&[child]);
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