mod individual;
mod partner;
mod gestation;
mod window;

use crate::individual::IndividualPlugin;
use crate::partner::PartnerPlugin;
use crate::gestation::GestationPlugin;
use crate::window::WindowPlugin;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
        
    app
        .add_plugin(IndividualPlugin)
        .add_plugin(PartnerPlugin)
        .add_plugin(GestationPlugin);

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