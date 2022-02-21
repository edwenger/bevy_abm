use bevy::prelude::*;
use rand::prelude::random;
use std::fmt::Formatter;

pub const GRID_WIDTH: u32 = 15;
pub const GRID_HEIGHT: u32 = 15;

const WINDOW_PIXEL_WIDTH: f32 = 800.0;
const WINDOW_PIXEL_HEIGHT: f32 = 800.0;

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
        .insert_resource(ClearColor(Color::rgb(0.74, 0.74, 0.74)))

        .add_startup_system(setup_camera)

        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

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