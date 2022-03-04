mod window;
mod individual;

use crate::window::WindowPlugin;
use crate::individual::IndividualPlugin;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
        
    app.add_plugin(IndividualPlugin);

    if cfg!(feature = "headless") {
        app
            .add_plugins(MinimalPlugins);
    } else {
        app
            .add_plugin(WindowPlugin)
            .add_plugins(DefaultPlugins);
    }

    app.run();
}