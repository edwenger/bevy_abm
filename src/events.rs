use bevy::prelude::*;
use std::fs::File;
use std::io::Write;

use crate::individual::{BirthEvent, DeathEvent};
use crate::partner::{BreakupEvent, PartnerEvent, WidowEvent};
use crate::config::Args;

#[derive(Resource, Default)]
pub struct EventLog {
    pub births: Vec<BirthEvent>,
    pub deaths: Vec<DeathEvent>,
    pub partnerships: Vec<PartnerEvent>,
    pub breakups: Vec<BreakupEvent>,
    pub widowings: Vec<WidowEvent>,
}

pub struct EventLogPlugin;

impl Plugin for EventLogPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EventLog>()
            .add_systems(Update, (
                log_birth_events,
                log_death_events,
                log_partner_events,
                log_breakup_events,
                log_widow_events,
            ))
            .add_systems(bevy::app::Last, print_event_summary);
    }
}

fn print_event_summary(
    event_log: Res<EventLog>,
    exit_events: EventReader<bevy::app::AppExit>,
    args: Res<Args>
) {
    if !exit_events.is_empty() {
        eprintln!("\n========== EVENT SUMMARY ==========");
        eprintln!("Births:       {}", event_log.births.len());
        eprintln!("Deaths:       {}", event_log.deaths.len());
        eprintln!("Partnerships: {}", event_log.partnerships.len());
        eprintln!("Breakups:     {}", event_log.breakups.len());
        eprintln!("Widowings:    {}", event_log.widowings.len());
        eprintln!("===================================\n");

        // Export to JSON if requested
        if args.export_events {
            export_births_json(&event_log);
        }
    }
}

fn export_births_json(event_log: &EventLog) {
    match serde_json::to_string_pretty(&event_log.births) {
        Ok(json) => {
            match File::create("births.json") {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        eprintln!("Error writing births.json: {}", e);
                    } else {
                        eprintln!("Exported {} birth events to births.json", event_log.births.len());
                    }
                }
                Err(e) => eprintln!("Error creating births.json: {}", e),
            }
        }
        Err(e) => eprintln!("Error serializing births to JSON: {}", e),
    }
}

fn log_birth_events(
    mut event_log: ResMut<EventLog>,
    mut events: EventReader<BirthEvent>
) {
    for event in events.read() {
        eprintln!("EVENT: Birth of {:?} (mother: {:?}) at time {:.2}",
            event.child_entity, event.mother_entity, event.time);
        event_log.births.push(BirthEvent {
            child_entity: event.child_entity,
            mother_entity: event.mother_entity,
            time: event.time,
        });
    }
}

fn log_death_events(
    mut event_log: ResMut<EventLog>,
    mut events: EventReader<DeathEvent>
) {
    for event in events.read() {
        eprintln!("EVENT: Death of {:?} at age {:.2}, time {:.2}",
            event.entity, event.age, event.time);
        event_log.deaths.push(DeathEvent {
            entity: event.entity,
            age: event.age,
            time: event.time,
        });
    }
}

fn log_partner_events(
    mut event_log: ResMut<EventLog>,
    mut events: EventReader<PartnerEvent>
) {
    for event in events.read() {
        eprintln!("EVENT: Partnership between {:?} and {:?} (rel: {:?}) at time {:.2}",
            event.individual1, event.individual2, event.relationship_entity, event.time);
        event_log.partnerships.push(PartnerEvent {
            individual1: event.individual1,
            individual2: event.individual2,
            relationship_entity: event.relationship_entity,
            time: event.time,
        });
    }
}

fn log_breakup_events(
    mut event_log: ResMut<EventLog>,
    mut events: EventReader<BreakupEvent>
) {
    for event in events.read() {
        eprintln!("EVENT: Breakup (male: {:?}, female: {:?}, rel: {:?}) at time {:.2}",
            event.male_entity, event._female_entity, event.relationship_entity, event.time);
        event_log.breakups.push(BreakupEvent {
            male_entity: event.male_entity,
            _female_entity: event._female_entity,
            relationship_entity: event.relationship_entity,
            time: event.time,
        });
    }
}

fn log_widow_events(
    mut event_log: ResMut<EventLog>,
    mut events: EventReader<WidowEvent>
) {
    for event in events.read() {
        eprintln!("EVENT: Widowing of {:?} (deceased: {:?}, rel: {:?}) at time {:.2}",
            event.widow_entity, event.deceased_entity, event.relationship_entity, event.time);
        event_log.widowings.push(WidowEvent {
            widow_entity: event.widow_entity,
            deceased_entity: event.deceased_entity,
            relationship_entity: event.relationship_entity,
            time: event.time,
        });
    }
}
