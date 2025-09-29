mod individual;
mod partner;
mod gestation;
mod window;

use crate::individual::IndividualPlugin;
use crate::partner::PartnerPlugin;
use crate::gestation::GestationPlugin;
use crate::window::{DisplayPlugin, WINDOW_PIXEL_WIDTH, WINDOW_PIXEL_HEIGHT};

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};

fn main() {
    let mut app = App::new();
        
    app
        .add_plugins((IndividualPlugin, PartnerPlugin, GestationPlugin));

    if cfg!(feature = "headless") {
        app
            .add_plugins(MinimalPlugins);
    } else {
        app
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ABM sandbox".to_string(),
                    resolution: (WINDOW_PIXEL_WIDTH, WINDOW_PIXEL_HEIGHT).into(),
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(DisplayPlugin);
    }

    app.run();
}