use crate::window::{Size, Position};

use bevy::prelude::*;

const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.1, 0.1, 0.4);
const FEMALE_COLOR: Color = Color::rgb(0.3, 0.1, 0.2);

#[derive(Debug)]
enum Sex {
    Male,
    Female,
}

impl Default for Sex {
    fn default() -> Self { Sex::Female }
}

#[derive(Component)]
struct Individual;

#[derive(Component, Default, Debug)]
pub struct Demog {
    age: f32,
    sex: Sex,
}

#[derive(Component)]
struct Gestation {
    remaining: f32,
}

#[derive(Component)]
struct PartnerSeeking;

pub fn get_older(time: Res<Time>, mut query: Query<(Entity, &mut Demog)> ) {
    for (e, mut demog) in query.iter_mut() {
        eprintln!("Entity {:?} is {}-year-old {:?}", e, demog.age, demog.sex);

        demog.age += 1.0;  // TODO: matching FixedTimestep size in system set should be enforced
    }
}

pub fn add_individual(mut commands: Commands) {
    let entity_id = commands
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
    println!("First individual");
}

