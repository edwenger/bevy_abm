mod window;
mod household;
mod individual;

use crate::window::WindowPlugin;
use crate::household::HouseholdPlugin;
use crate::individual::{add_individual, get_older};

use bevy::prelude::*;
use bevy::core::FixedTimestep;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_startup_system(add_individual)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(get_older),
        )
        // .add_plugin(HouseholdPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}