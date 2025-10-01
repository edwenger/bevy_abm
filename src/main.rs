mod individual;
mod partner;
mod gestation;
mod window;
mod config;
mod events;

use crate::individual::IndividualPlugin;
use crate::partner::PartnerPlugin;
use crate::gestation::GestationPlugin;
use crate::window::{DisplayPlugin, WINDOW_PIXEL_WIDTH, WINDOW_PIXEL_HEIGHT};
use crate::config::{ConfigPlugin, Args};
use crate::events::EventLogPlugin;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use clap::Parser;

fn main() {
    let args = Args::parse();
    let mut app = App::new();

    app
        .insert_resource(args)
        .add_plugins((IndividualPlugin, PartnerPlugin, GestationPlugin, ConfigPlugin, EventLogPlugin));

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