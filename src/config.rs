use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug, Resource)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Initial population size for headless mode
    #[arg(short = 'n', long, default_value_t = 0)]
    pub initial_population: usize,

    /// Simulation duration in years (for headless mode)
    #[arg(short = 's', long)]
    pub sim_years: Option<f32>,
}

#[derive(Resource, Clone)]
pub struct SimulationParameters {
    // Demographics - ages and rates, not timesteps
    pub death_age: f32,
    pub min_partner_seeking_age: f32,
    pub max_partner_seeking_age: f32,
    pub spawn_individual_age: f32,

    // Reproduction - rates and durations, not timesteps
    pub min_conception_age: f32,
    pub max_conception_age: f32,
    pub conception_rate: f32,
    pub gestation_duration: f32,

    // Relationships - rates, not timesteps
    pub breakup_rate: f32,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            // Demographics - ages only
            death_age: 30.0,
            min_partner_seeking_age: 20.0,
            max_partner_seeking_age: 50.0,
            spawn_individual_age: 18.0,

            // Reproduction - rates and durations only
            min_conception_age: 25.0,
            max_conception_age: 35.0,
            conception_rate: 0.5,
            gestation_duration: 40.0 / 52.0,

            // Relationships - rates only
            breakup_rate: 0.1, // low rate for relationship stability
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationParameters>();
    }
}