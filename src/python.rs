use pyo3::prelude::*;
use pyo3::types::PyDict;

use bevy::prelude::*;

use crate::individual::IndividualPlugin;
use crate::partner::PartnerPlugin;
use crate::gestation::GestationPlugin;
use crate::config::{ConfigPlugin, Args, SimulationParameters};
use crate::events::EventLogPlugin;

/// Run a demographic simulation with given parameters
///
/// Parameters:
/// - params: dict with simulation parameters (initial_population, sim_years, etc.)
///
/// Example:
/// >>> import pybevy_demog
/// >>> pybevy_demog.run_simulation({
/// ...     "initial_population": 50,
/// ...     "sim_years": 10.0,
/// ...     "death_age": 70.0,
/// ...     "conception_rate": 0.5
/// ... })
#[pyfunction]
fn run_simulation(params: &Bound<'_, PyDict>) -> PyResult<()> {
    // Extract parameters from Python dict
    let initial_population = params.get_item("initial_population")?
        .and_then(|v| v.extract::<usize>().ok())
        .unwrap_or(0);

    let sim_years = params.get_item("sim_years")?
        .and_then(|v| v.extract::<f32>().ok());

    let export_events = params.get_item("export_events")?
        .and_then(|v| v.extract::<bool>().ok())
        .unwrap_or(false);

    // Build Args resource
    let args = Args {
        initial_population,
        sim_years,
        export_events,
    };

    // Build SimulationParameters with defaults, overriding from dict
    let mut sim_params = SimulationParameters::default();

    if let Some(Ok(death_age)) = params.get_item("death_age")?.map(|v| v.extract::<f32>()) {
        sim_params.death_age = death_age;
    }
    if let Some(Ok(min_partner_seeking_age)) = params.get_item("min_partner_seeking_age")?.map(|v| v.extract::<f32>()) {
        sim_params.min_partner_seeking_age = min_partner_seeking_age;
    }
    if let Some(Ok(max_partner_seeking_age)) = params.get_item("max_partner_seeking_age")?.map(|v| v.extract::<f32>()) {
        sim_params.max_partner_seeking_age = max_partner_seeking_age;
    }
    if let Some(Ok(min_conception_age)) = params.get_item("min_conception_age")?.map(|v| v.extract::<f32>()) {
        sim_params.min_conception_age = min_conception_age;
    }
    if let Some(Ok(max_conception_age)) = params.get_item("max_conception_age")?.map(|v| v.extract::<f32>()) {
        sim_params.max_conception_age = max_conception_age;
    }
    if let Some(Ok(conception_rate)) = params.get_item("conception_rate")?.map(|v| v.extract::<f32>()) {
        sim_params.conception_rate = conception_rate;
    }
    if let Some(Ok(gestation_duration)) = params.get_item("gestation_duration")?.map(|v| v.extract::<f32>()) {
        sim_params.gestation_duration = gestation_duration;
    }
    if let Some(Ok(breakup_rate)) = params.get_item("breakup_rate")?.map(|v| v.extract::<f32>()) {
        sim_params.breakup_rate = breakup_rate;
    }

    // Create and run headless simulation with minimal logging
    let mut app = App::new();

    // Use a static to track if we've already set up logging
    use std::sync::atomic::{AtomicBool, Ordering};
    static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);

    app
        .insert_resource(args)
        .insert_resource(sim_params)
        .add_plugins(MinimalPlugins);

    // Only add LogPlugin on first simulation run
    if !LOGGING_INITIALIZED.swap(true, Ordering::SeqCst) {
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::WARN,
            filter: "wgpu=error,bevy_render=error,bevy_ecs=warn".to_string(),
            update_subscriber: None,
        });
    }

    app.add_plugins((
        IndividualPlugin,
        PartnerPlugin,
        GestationPlugin,
        ConfigPlugin,
        EventLogPlugin
    ));

    // Run the simulation
    app.run();

    Ok(())
}

/// Python module definition
#[pymodule]
fn pybevy_demog(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_simulation, m)?)?;
    Ok(())
}
