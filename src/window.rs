use bevy::prelude::*;
use bevy::window::{Window};
use bevy::input::{ButtonInput};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use rand::prelude::random;
use std::fmt::Formatter;

use crate::individual::{
    Individual, Demog, Adult, Elder, Sex, spawn_individual, BirthEvent
};
use crate::partner::{
    Partner, PartnerSeeking, Relationship, Partners, BreakupEvent
};
use crate::gestation::Mother;
use crate::config::SimulationParameters;

pub const GRID_WIDTH: u32 = 15;
pub const GRID_HEIGHT: u32 = 15;

pub const WINDOW_PIXEL_WIDTH: f32 = 800.0;
pub const WINDOW_PIXEL_HEIGHT: f32 = 800.0;

// SPAWN_INDIVIDUAL_AGE now comes from SimulationParameters

//-- DISPLAY
const CHILD_COLOR: Color = Color::rgb(0.6, 0.6, 0.6);
const MALE_COLOR: Color = Color::rgb(0.2, 0.4, 0.6);
const FEMALE_COLOR: Color = Color::rgb(0.5, 0.2, 0.4);
const MIN_SPRITE_SIZE: f32 = 0.05;
const MAX_SPRITE_SIZE: f32 = 0.3;
const MOVE_VELOCITY: f32 = 5.0;
const PARTNER_DESTINATION_RANDOM_SCALE: f32 = 5.0;

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))

        .add_systems(Startup, setup_camera)
        .add_systems(Update, (
            keyboard_input,
            camera_controls,
            display_new_individual,
            update_child_size,
            assign_new_adult_color,
            assign_elder_color,
            assign_pair_destination,
            handle_breakup_movement,
            move_towards,
            simulation_controls_ui,
        ))
        .add_systems(PostUpdate, (
            position_translation,
            size_scaling,
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn keyboard_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    params: Res<SimulationParameters>,
    mut birth_events: EventWriter<BirthEvent>,
    time: Res<Time>
) {
    if keys.just_pressed(KeyCode::Enter) {
        // Return was pressed --> add a random person
        spawn_individual(&mut commands, params.spawn_individual_age, None, &mut birth_events, &time);
    }
}

fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>
) {
    const CAMERA_SPEED: f32 = 500.0; // pixels per second

    if let Ok(mut transform) = camera_query.get_single_mut() {
        let mut direction = Vec2::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            direction = direction.normalize();
            let movement = direction * CAMERA_SPEED * time.delta_seconds();
            transform.translation += Vec3::new(movement.x, movement.y, 0.0);
        }
    }
}

// ------ SPAWNING POSITION ------

fn calculate_spawn_position(
    mother_position: Option<&Position>,
    camera_query: &Query<&Transform, With<Camera2d>>
) -> Position {
    // If there's a mother position, spawn near her
    if let Some(mother_pos) = mother_position {
        return position_near_parent(mother_pos);
    }

    // Try to spawn in camera view if camera exists
    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_pos = camera_transform.translation;

        // Convert camera world position to grid coordinates
        // This reverses the conversion done in position_translation
        let grid_x = (camera_pos.x + (WINDOW_PIXEL_WIDTH / 2.0)) / (WINDOW_PIXEL_WIDTH / GRID_WIDTH as f32);
        let grid_y = (camera_pos.y + (WINDOW_PIXEL_HEIGHT / 2.0)) / (WINDOW_PIXEL_HEIGHT / GRID_HEIGHT as f32);

        // Spawn randomly within the currently visible grid area
        let spawn_x = grid_x + (random::<f32>() - 0.5) * GRID_WIDTH as f32;
        let spawn_y = grid_y + (random::<f32>() - 0.5) * GRID_HEIGHT as f32;

        // Clamp to ensure we stay within valid grid bounds
        let spawn_x = spawn_x.clamp(0.0, GRID_WIDTH as f32);
        let spawn_y = spawn_y.clamp(0.0, GRID_HEIGHT as f32);

        return Position(Vec2::new(spawn_x, spawn_y));
    }

    // Fallback to random position anywhere on grid
    Position::random_cell()
}

// ------ DISPLAY ------

#[derive(Component)]
pub struct MovingTowards(Position);

pub fn display_new_individual(
    mut commands: Commands,
    query: Query<(Entity, &Demog, Option<&Mother>), Added<Individual>>,
    mother_query: Query<&Position>,
    camera_query: Query<&Transform, With<Camera2d>>,
    params: Res<SimulationParameters>
) {
    for (e, demog, mother_opt) in query.iter() {
        let color = if demog.age < params.min_partner_seeking_age {
            CHILD_COLOR
        } else {
            color_for_sex(demog.sex)
        };

        commands
            .entity(e)
            .insert(SpriteBundle {
                sprite: Sprite {
                    color: color,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Size::square(size_for_age(demog.age, params.min_partner_seeking_age)));

        // Calculate spawn position based on mother location and camera view
        let mother_position = mother_opt
            .and_then(|mother| mother_query.get(mother.0).ok());
        let position = calculate_spawn_position(mother_position, &camera_query);
        commands.entity(e).insert(position);

    }
}

fn color_for_sex(sex: Sex) -> Color {
    return if sex==Sex::Female {
        FEMALE_COLOR
    } else {
        MALE_COLOR
    }
}

fn size_for_age(age: f32, min_partner_seeking_age: f32) -> f32 {
    return MIN_SPRITE_SIZE + (MAX_SPRITE_SIZE - MIN_SPRITE_SIZE) * age / min_partner_seeking_age;
}

fn position_near_parent(p: &Position) -> Position {
    Position(p.0 + Vec2::new(
        random::<f32>() - 0.5,
        random::<f32>() - 0.5
    ))
}

pub fn update_child_size(
    mut query: Query<(&Demog, &mut Size), Without<Adult>>,
    params: Res<SimulationParameters>
) {
    for (demog, mut size) in query.iter_mut() {
        size.resize(size_for_age(demog.age, params.min_partner_seeking_age));
    }
}

pub fn assign_new_adult_color(
    mut query: Query<(&Demog, &mut Sprite), Added<Adult>>,
) {
    for (demog, mut sprite) in query.iter_mut() {
        sprite.color = color_for_sex(demog.sex);
    }
}

pub fn assign_elder_color(
    mut query: Query<(&Demog, &mut Sprite), Added<Elder>>,
) {
    for (demog, mut sprite) in query.iter_mut() {
        let base_color = color_for_sex(demog.sex);
        // Darken the color by reducing RGB values to 80%
        sprite.color = Color::rgb(
            base_color.r() * 0.8,
            base_color.g() * 0.8,
            base_color.b() * 0.8
        );
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

                let destination = Position(midpoint.0 + Vec2::new(
                    PARTNER_DESTINATION_RANDOM_SCALE * (random::<f32>() - 0.5),
                    PARTNER_DESTINATION_RANDOM_SCALE * (random::<f32>() - 0.5),
                ));
                
                commands.entity(partners.e1).insert(MovingTowards(destination));
                commands.entity(partners.e2).insert(MovingTowards(destination));
            } 
        }
    }
}

pub fn handle_breakup_movement(
    mut commands: Commands,
    mut breakup_events: EventReader<BreakupEvent>,
    position_query: Query<&Position>
) {
    for event in breakup_events.read() {
        if let Ok(male_position) = position_query.get(event.male_entity) {
            // Move male away from current position
            let move_distance = GRID_WIDTH as f32 * 0.05; // 5% of grid width
            let random_direction = Vec2::new(
                random::<f32>() - 0.5,
                random::<f32>() - 0.5
            ).normalize();
            let destination = Position(male_position.0 + random_direction * move_distance);

            commands.entity(event.male_entity).insert(MovingTowards(destination));
            eprintln!("Male {:?} moving away after breakup at time {:.1}", event.male_entity, event.time);
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
            pos.0 = pos.0 + u.0 * v;
        } else {
            commands.entity(e).remove::<MovingTowards>();
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Position(pub Vec2);

impl Position {
    pub fn random_cell() -> Self {
        Self(Vec2::new(
            random::<f32>() * GRID_WIDTH as f32,
            random::<f32>() * GRID_HEIGHT as f32,
        ))
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.0.distance(other.0)
    }

    pub fn unit_direction(&self, other: &Self) -> Self {
        let direction = (other.0 - self.0).normalize();
        Self(direction)
    }

    pub fn midpoint(&self, other: &Self) -> Self {
        Self((self.0 + other.0) * 0.5)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "({:.1}, {:.1})", self.0.x, self.0.y)
    }
}

fn position_translation(window_query: Query<&Window>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    if let Ok(window) = window_query.get_single() {
        for (pos, mut transform) in q.iter_mut() {
            transform.translation = Vec3::new(
                convert(pos.0.x, window.resolution.width(), GRID_WIDTH as f32),
                convert(pos.0.y, window.resolution.height(), GRID_HEIGHT as f32),
                0.0,
            );
        }
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

fn size_scaling(window_query: Query<&Window>, mut q: Query<(&Size, &mut Transform)>) {
    if let Ok(window) = window_query.get_single() {
        for (sprite_size, mut transform) in q.iter_mut() {
            transform.scale = Vec3::new(
                sprite_size.width / GRID_WIDTH as f32 * window.resolution.width(),
                sprite_size.height / GRID_HEIGHT as f32 * window.resolution.height(),
                1.0,
            );
        }
    }
}

fn simulation_controls_ui(
    mut contexts: EguiContexts,
    mut params: ResMut<SimulationParameters>
) {
    egui::Window::new("ABM Sandbox Controls")
        .default_pos(egui::pos2(10.0, 10.0))
        .default_size(egui::vec2(300.0, 400.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.label("🎮 Controls:");
            ui.label("• Press ENTER to spawn new individual");
            ui.label("• Use WASD to move camera");
            ui.label("• Use sliders to adjust parameters");
            ui.label("• Watch demographics evolve!");

            ui.separator();

            ui.heading("Simulation Parameters");

            // Death Age slider
            ui.label("Death Age");
            let response = ui.add(egui::Slider::new(&mut params.death_age, 20.0..=100.0).text("years"));
            if response.changed() {
                eprintln!("Death age changed to: {}", params.death_age);
            }

            ui.separator();

            // Min Partner Seeking Age slider
            ui.label("Min Partner Seeking Age");
            let response = ui.add(egui::Slider::new(&mut params.min_partner_seeking_age, 15.0..=30.0).text("years"));
            if response.changed() {
                eprintln!("Min partner seeking age changed to: {}", params.min_partner_seeking_age);
            }

            // Max Partner Seeking Age slider
            ui.label("Max Partner Seeking Age");
            let response = ui.add(egui::Slider::new(&mut params.max_partner_seeking_age, 40.0..=70.0).text("years"));
            if response.changed() {
                eprintln!("Max partner seeking age changed to: {}", params.max_partner_seeking_age);
            }

            ui.separator();

            // Spawn Individual Age slider
            ui.label("Spawn Individual Age");
            let response = ui.add(egui::Slider::new(&mut params.spawn_individual_age, 15.0..=25.0).text("years"));
            if response.changed() {
                eprintln!("Spawn individual age changed to: {}", params.spawn_individual_age);
            }

            ui.separator();

            // Conception Rate slider
            ui.label("Conception Rate");
            let response = ui.add(egui::Slider::new(&mut params.conception_rate, 0.1..=2.0).text("rate"));
            if response.changed() {
                eprintln!("Conception rate changed to: {}", params.conception_rate);
            }

            ui.separator();

            // Min Conception Age slider
            ui.label("Min Conception Age");
            let response = ui.add(egui::Slider::new(&mut params.min_conception_age, 18.0..=35.0).text("years"));
            if response.changed() {
                eprintln!("Min conception age changed to: {}", params.min_conception_age);
            }

            // Max Conception Age slider
            ui.label("Max Conception Age");
            let response = ui.add(egui::Slider::new(&mut params.max_conception_age, 25.0..=50.0).text("years"));
            if response.changed() {
                eprintln!("Max conception age changed to: {}", params.max_conception_age);
            }

            ui.separator();

            // Gestation Duration slider
            ui.label("Gestation Duration");
            let response = ui.add(egui::Slider::new(&mut params.gestation_duration, 0.5..=1.5).text("time units"));
            if response.changed() {
                eprintln!("Gestation duration changed to: {}", params.gestation_duration);
            }

            ui.separator();

            // Breakup Rate slider
            ui.label("Breakup Rate");
            let response = ui.add(egui::Slider::new(&mut params.breakup_rate, 0.0..=1.0).text("per year"));
            if response.changed() {
                eprintln!("Breakup rate changed to: {}", params.breakup_rate);
            }
        });
}