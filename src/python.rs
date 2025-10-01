use pyo3::prelude::*;
use pyo3::types::PyDict;

use bevy::prelude::*;
use polars::prelude::*;
use pyo3_polars::PyDataFrame;
use std::sync::{Arc, Mutex};

use crate::individual::IndividualPlugin;
use crate::partner::PartnerPlugin;
use crate::gestation::GestationPlugin;
use crate::config::{ConfigPlugin, Args, SimulationParameters};
use crate::events::{EventLogPlugin, EventLog};

/// Resource to capture EventLog before app exits
#[derive(Resource)]
struct EventLogCapture {
    captured: Arc<Mutex<Option<EventLog>>>,
}

/// System to capture EventLog on exit
fn capture_event_log_on_exit(
    event_log: Res<EventLog>,
    exit_events: EventReader<bevy::app::AppExit>,
    capture: Res<EventLogCapture>,
) {
    if !exit_events.is_empty() {
        // Take ownership of the event log data (move it out)
        let log_data = EventLog {
            births: event_log.births.clone(),
            deaths: event_log.deaths.clone(),
            partnerships: event_log.partnerships.clone(),
            breakups: event_log.breakups.clone(),
            widowings: event_log.widowings.clone(),
        };
        *capture.captured.lock().unwrap() = Some(log_data);
    }
}

/// Convert birth events to polars DataFrame
fn events_to_births_dataframe(event_log: &EventLog) -> PyResult<DataFrame> {
    let child_entities: Vec<u64> = event_log.births.iter()
        .map(|e| e.child_entity.to_bits())
        .collect();
    let mother_entities: Vec<Option<u64>> = event_log.births.iter()
        .map(|e| e.mother_entity.map(|m| m.to_bits()))
        .collect();
    let times: Vec<f32> = event_log.births.iter()
        .map(|e| e.time)
        .collect();

    DataFrame::new(vec![
        Series::new("child_entity".into(), child_entities),
        Series::new("mother_entity".into(), mother_entities),
        Series::new("time".into(), times),
    ]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Polars error: {}", e)))
}

/// Convert death events to polars DataFrame
fn events_to_deaths_dataframe(event_log: &EventLog) -> PyResult<DataFrame> {
    let entities: Vec<u64> = event_log.deaths.iter()
        .map(|e| e.entity.to_bits())
        .collect();
    let ages: Vec<f32> = event_log.deaths.iter()
        .map(|e| e.age)
        .collect();
    let times: Vec<f32> = event_log.deaths.iter()
        .map(|e| e.time)
        .collect();

    DataFrame::new(vec![
        Series::new("entity".into(), entities),
        Series::new("age".into(), ages),
        Series::new("time".into(), times),
    ]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Polars error: {}", e)))
}

/// Convert partnership events to polars DataFrame
fn events_to_partnerships_dataframe(event_log: &EventLog) -> PyResult<DataFrame> {
    let individual1s: Vec<u64> = event_log.partnerships.iter()
        .map(|e| e.individual1.to_bits())
        .collect();
    let individual2s: Vec<u64> = event_log.partnerships.iter()
        .map(|e| e.individual2.to_bits())
        .collect();
    let relationship_entities: Vec<u64> = event_log.partnerships.iter()
        .map(|e| e.relationship_entity.to_bits())
        .collect();
    let times: Vec<f32> = event_log.partnerships.iter()
        .map(|e| e.time)
        .collect();

    DataFrame::new(vec![
        Series::new("individual1".into(), individual1s),
        Series::new("individual2".into(), individual2s),
        Series::new("relationship_entity".into(), relationship_entities),
        Series::new("time".into(), times),
    ]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Polars error: {}", e)))
}

/// Convert breakup events to polars DataFrame
fn events_to_breakups_dataframe(event_log: &EventLog) -> PyResult<DataFrame> {
    let male_entities: Vec<u64> = event_log.breakups.iter()
        .map(|e| e.male_entity.to_bits())
        .collect();
    let female_entities: Vec<u64> = event_log.breakups.iter()
        .map(|e| e._female_entity.to_bits())
        .collect();
    let relationship_entities: Vec<u64> = event_log.breakups.iter()
        .map(|e| e.relationship_entity.to_bits())
        .collect();
    let times: Vec<f32> = event_log.breakups.iter()
        .map(|e| e.time)
        .collect();

    DataFrame::new(vec![
        Series::new("male_entity".into(), male_entities),
        Series::new("female_entity".into(), female_entities),
        Series::new("relationship_entity".into(), relationship_entities),
        Series::new("time".into(), times),
    ]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Polars error: {}", e)))
}

/// Convert widowing events to polars DataFrame
fn events_to_widowings_dataframe(event_log: &EventLog) -> PyResult<DataFrame> {
    let widow_entities: Vec<u64> = event_log.widowings.iter()
        .map(|e| e.widow_entity.to_bits())
        .collect();
    let deceased_entities: Vec<u64> = event_log.widowings.iter()
        .map(|e| e.deceased_entity.to_bits())
        .collect();
    let relationship_entities: Vec<u64> = event_log.widowings.iter()
        .map(|e| e.relationship_entity.to_bits())
        .collect();
    let times: Vec<f32> = event_log.widowings.iter()
        .map(|e| e.time)
        .collect();

    DataFrame::new(vec![
        Series::new("widow_entity".into(), widow_entities),
        Series::new("deceased_entity".into(), deceased_entities),
        Series::new("relationship_entity".into(), relationship_entities),
        Series::new("time".into(), times),
    ]).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Polars error: {}", e)))
}

/// Run a demographic simulation with given parameters
///
/// Parameters:
/// - params: dict with simulation parameters (initial_population, sim_years, etc.)
///
/// Returns:
/// - dict with polars DataFrames: {"births": df, "deaths": df, "partnerships": df, "breakups": df, "widowings": df}
///
/// Example:
/// >>> import pybevy_demog
/// >>> results = pybevy_demog.run_simulation({
/// ...     "initial_population": 50,
/// ...     "sim_years": 10.0,
/// ...     "death_age": 70.0,
/// ...     "conception_rate": 0.5
/// ... })
/// >>> print(results["births"])
#[pyfunction]
fn run_simulation(py: Python, params: &Bound<'_, PyDict>) -> PyResult<Py<PyDict>> {
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

    // Create capture resource to extract EventLog after simulation
    let capture = Arc::new(Mutex::new(None));
    let capture_resource = EventLogCapture {
        captured: capture.clone(),
    };

    app
        .insert_resource(args)
        .insert_resource(sim_params)
        .insert_resource(capture_resource)
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
    ))
    .add_systems(bevy::app::Last, capture_event_log_on_exit);

    // Run the simulation
    app.run();

    // Extract captured event log
    let event_log = capture.lock().unwrap()
        .take()
        .expect("EventLog should have been captured on exit");

    // Convert events to DataFrames
    let births_df = events_to_births_dataframe(&event_log)?;
    let deaths_df = events_to_deaths_dataframe(&event_log)?;
    let partnerships_df = events_to_partnerships_dataframe(&event_log)?;
    let breakups_df = events_to_breakups_dataframe(&event_log)?;
    let widowings_df = events_to_widowings_dataframe(&event_log)?;

    // Create Python dict with DataFrames
    let result = PyDict::new_bound(py);
    result.set_item("births", PyDataFrame(births_df).into_py(py))?;
    result.set_item("deaths", PyDataFrame(deaths_df).into_py(py))?;
    result.set_item("partnerships", PyDataFrame(partnerships_df).into_py(py))?;
    result.set_item("breakups", PyDataFrame(breakups_df).into_py(py))?;
    result.set_item("widowings", PyDataFrame(widowings_df).into_py(py))?;

    Ok(result.into())
}

/// Python module definition
#[pymodule]
fn bevy_abm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_simulation, m)?)?;
    Ok(())
}
