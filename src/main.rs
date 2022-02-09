use bevy::prelude::*;
use rand::prelude::random;
use bevy::core::FixedTimestep;

const GRID_WIDTH: u32 = 15;
const GRID_HEIGHT: u32 = 15;

const WINDOW_PIXEL_WIDTH: f32 = 800.0;
const WINDOW_PIXEL_HEIGHT: f32 = 800.0;

const HOUSEHOLD_COLOR: Color = Color::rgb(0.1, 0.4, 0.2);

#[derive(Component)]
struct Household;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
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

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn household_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: HOUSEHOLD_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Household)
        .insert(Position {
            x: (random::<f32>() * GRID_WIDTH as f32) as i32,
            y: (random::<f32>() * GRID_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "ABM sandbox".to_string(),
            width: WINDOW_PIXEL_WIDTH,
            height: WINDOW_PIXEL_HEIGHT,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup_camera)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(household_spawner),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}