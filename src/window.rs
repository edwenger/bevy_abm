use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera2d, FlyCameraPlugin};

use rand::prelude::random;
use std::fmt::Formatter;

use crate::individual::{
    Individual, Demog, Adult, Sex, spawn_individual
};
use crate::partner::{
    PARTNER_SEEKING_AGE,
    Partner, PartnerSeeking, Relationship, Partners
};
use crate::gestation::Mother;

pub const GRID_WIDTH: u32 = 15;
pub const GRID_HEIGHT: u32 = 15;

const WINDOW_PIXEL_WIDTH: f32 = 800.0;
const WINDOW_PIXEL_HEIGHT: f32 = 800.0;

const SPAWN_INDIVIDUAL_AGE: f32 = 18.0;

//-- DISPLAY
const CHILD_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);
const MIN_SPRITE_SIZE: f32 = 0.05;
const MAX_SPRITE_SIZE: f32 = 0.3;
const MOVE_VELOCITY: f32 = 5.0;
const PARTNER_DESTINATION_RANDOM_SCALE: f32 = 5.0;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(WindowDescriptor {
            title: "ABM sandbox".to_string(),
            width: WINDOW_PIXEL_WIDTH,
            height: WINDOW_PIXEL_HEIGHT,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))

        .add_startup_system(setup_camera)
        .add_plugin(FlyCameraPlugin)

        .add_system(keyboard_input)

        //-- DISPLAY
        .add_system(display_new_individual)
        .add_system(update_child_size)
        .add_system(assign_new_adult_color)
        .add_system(assign_pair_destination)
        .add_system(move_towards)

        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(FlyCamera2d::default());
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        // Return was pressed --> add a random person
        spawn_individual(&mut commands, SPAWN_INDIVIDUAL_AGE, None);
    }
}

// ------ DISPLAY ------

#[derive(Component)]
pub struct MovingTowards(Position);

pub fn display_new_individual(
    mut commands: Commands,
    query: Query<(Entity, &Demog, Option<&Mother>), Added<Individual>>,
    mother_query: Query<&Position>
) {
    for (e, demog, mother_opt) in query.iter() {
        let color = if demog.age < PARTNER_SEEKING_AGE {
            CHILD_COLOR
        } else {
            color_for_sex(demog.sex)
        };

        commands
            .entity(e)
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: color,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Size::square(size_for_age(demog.age)))
            .id();

        if let Some(mother) = mother_opt {
            // TODO: cleaner syntax for checking if has Mother with Position?
            // https://github.com/rust-lang/rfcs/blob/master/text/2497-if-let-chains.md#chained-if-lets-inside-match-arms
            if let Ok(mother_position) = mother_query.get(mother.0) {
                commands.entity(e).insert(position_near_parent(mother_position));
            } else {
                commands.entity(e).insert(Position::random_cell());
            }
        } else {
            commands.entity(e).insert(Position::random_cell());
        }

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

fn position_near_parent(p: &Position) -> Position {
    return Position{
        x: p.x - 0.5 + random::<f32>(),
        y: p.y - 0.5 + random::<f32>()
    }
}

pub fn update_child_size(mut query: Query<(&Demog, &mut Size), Without<Adult>>) {
    for (demog, mut size) in query.iter_mut() {
        size.resize(size_for_age(demog.age));
    }
}

pub fn assign_new_adult_color(
    mut query: Query<(&Demog, &mut Sprite), Added<Adult>>,
) {
    for (demog, mut sprite) in query.iter_mut() {
        sprite.color = color_for_sex(demog.sex);
    }
}

pub fn assign_pair_destination(
    mut commands: Commands,
    rel_query: Query<&Partners, Added<Relationship>>,
    ind_query: Query<(&Individual, &Position), (Without<Partner>, With<PartnerSeeking>)>
) {
    for partners in rel_query.iter() {
        if let Ok((_ind1, pos1)) = ind_query.get(partners.e1) {
            if let Ok((_ind2, pos2)) = ind_query.get(partners.e2) {

                let midpoint = pos1.midpoint(pos2);

                let destination = Position {
                    x: midpoint.x + PARTNER_DESTINATION_RANDOM_SCALE * (random::<f32>() - 0.5),
                    y: midpoint.y + PARTNER_DESTINATION_RANDOM_SCALE * (random::<f32>() - 0.5),
                };
                
                commands.entity(partners.e1).insert(MovingTowards(destination));
                commands.entity(partners.e2).insert(MovingTowards(destination));
            } 
        }
    }
}

pub fn move_towards(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Position, &MovingTowards)>
) {
    for (e, mut pos, destination) in query.iter_mut() {
        let distance = pos.distance(&destination.0);
        if distance > MAX_SPRITE_SIZE * 0.7071 {  // almost touching on diagonal
            let v = MOVE_VELOCITY * time.delta_seconds();
            let u = pos.unit_direction(&destination.0);
            pos.x = pos.x + u.x * v;
            pos.y = pos.y + u.y * v;
        } else {
            commands.entity(e).remove::<MovingTowards>();
        }
    }
}

// TODO: use Vec2 class for distance, speed, unit vector operations
//   - Position is component, but could hold (or impl) Vec2
//   - Formatter, +/-/* operator, etc.


#[derive(Component, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn random_cell() -> Self {
        Self {
            x: random::<f32>() * GRID_WIDTH as f32,
            y: random::<f32>() * GRID_HEIGHT as f32,
        }
    }
    pub fn distance(&self, other: &Self) -> f32 {
        return ((self.y - other.y).powf(2.0) + (self.x - other.x).powf(2.0)).sqrt();
    }
    pub fn unit_direction(&self, other: &Self) -> Self {
        let distance = self.distance(other);
        Self {
            x: (other.x - self.x) / distance,
            y: (other.y - self.y) / distance
        }
    }
    pub fn midpoint(&self, other: &Self) -> Self {
        Self {
            x: self.x + (other.x - self.x) / 2.0,
            y: self.y + (other.y - self.y) / 2.0
        }
    }
}
impl std::fmt::Display for Position {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO: look up API of Formatter
        Ok(())
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GRID_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, GRID_HEIGHT as f32),
            0.0,
        );
    }
}

#[derive(Component)]
pub struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
    pub fn resize(&mut self, x: f32) {
        self.width = x;
        self.height = x;
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / GRID_WIDTH as f32 * window.width() as f32,
            sprite_size.height / GRID_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}