#[macro_use]
extern crate approx;

use bevy::prelude::*;
use bevy::ecs::event::Events;

use bevy_abm::individual::{Individual, Demog, Sex, Adult, BirthEvent};
use bevy_abm::gestation::{RemainingGestation, update_gestation, Mother};
use bevy_abm::config::SimulationParameters;

#[test]
fn test_gestation_completes_after_correct_timesteps() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0,
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        min_conception_age: 22.0,
        max_conception_age: 40.0,
        conception_rate: 1.0,
        gestation_duration: 1.0, // 1 time unit
        ..Default::default()
    };
    world.insert_resource(params.clone());
    world.init_resource::<Events<BirthEvent>>();
    world.init_resource::<Time>();

    // Create pregnant female
    let mother_entity = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Female },
        RemainingGestation(params.gestation_duration)
    )).id();

    // Setup gestation system
    let mut gestation_schedule = Schedule::default();
    gestation_schedule.add_systems(update_gestation);

    // Check initial state
    assert!(world.get::<RemainingGestation>(mother_entity).is_some(), "Mother should have RemainingGestation");
    let initial_individual_count = world.query::<&Individual>().iter(&world).count();
    assert_eq!(initial_individual_count, 1, "Should have 1 individual initially");

    // Calculate expected timesteps for completion
    // CONCEPTION_TIMESTEP = 1.0/52.0 (from gestation.rs)
    let conception_timestep = 1.0/52.0;
    let expected_timesteps = (params.gestation_duration / conception_timestep).ceil() as i32;

    // Run gestation updates until birth occurs
    let mut timesteps_run = 0;
    for i in 0..100 { // safety limit
        timesteps_run = i + 1;
        gestation_schedule.run(&mut world);

        // Check if birth occurred (RemainingGestation removed from mother)
        if world.get::<RemainingGestation>(mother_entity).is_none() {
            break;
        }
    }

    // Verify birth occurred at expected time
    println!("Expected timesteps: {}, Actual timesteps: {}", expected_timesteps, timesteps_run);
    assert!(timesteps_run <= expected_timesteps + 1, "Birth should occur within expected timestep range");
    assert!(timesteps_run >= expected_timesteps - 1, "Birth should not occur too early");

    // Verify mother no longer has RemainingGestation
    assert!(world.get::<RemainingGestation>(mother_entity).is_none(), "Mother should no longer have RemainingGestation");

    // Verify a new individual was created (newborn)
    let final_individual_count = world.query::<&Individual>().iter(&world).count();
    assert_eq!(final_individual_count, 2, "Should have 2 individuals after birth (mother + newborn)");

    // Find the newborn (not the mother entity)
    let mut newborn_entity = None;
    let mut newborn_age = None;
    let mut newborn_mother = None;

    for (entity, demog, mother_opt) in world.query::<(Entity, &Demog, Option<&Mother>)>().iter(&world) {
        if entity != mother_entity {
            newborn_entity = Some(entity);
            newborn_age = Some(demog.age);
            newborn_mother = mother_opt.map(|m| m.0);
        }
    }

    // Verify newborn properties
    assert!(newborn_entity.is_some(), "Should have found newborn entity");

    let newborn_age = newborn_age.expect("Newborn should have age");
    assert!(relative_eq!(newborn_age, 0.0, epsilon = 0.001), "Newborn should have approximately zero age, got {}", newborn_age);

    let newborn_mother_id = newborn_mother.expect("Newborn should have Mother component");
    assert_eq!(newborn_mother_id, mother_entity, "Newborn's Mother component should reference mother entity");

    // Verify newborn has Individual component
    let newborn_id = newborn_entity.unwrap();
    assert!(world.get::<Individual>(newborn_id).is_some(), "Newborn should have Individual component");
}