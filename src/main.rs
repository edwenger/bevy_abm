mod window;
mod individual;

use crate::window::WindowPlugin;
use crate::individual::IndividualPlugin;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugin(WindowPlugin)
        .add_plugin(IndividualPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}