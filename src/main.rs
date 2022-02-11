mod window;
// mod household;
mod individual;

use crate::window::WindowPlugin;
// use crate::household::HouseholdPlugin;
use crate::individual::IndividualPlugin;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugin(IndividualPlugin)
        // .add_plugin(HouseholdPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}