use bevy::prelude::*;

use bevy_abm::individual::{Individual, Demog, Sex, Adult};
use bevy_abm::partner::{Partner, start_partner_seeking,
                        queue_partner_seekers, match_partners, resolve_matches,
                        random_breakups, detect_widows, AvailableSeekers,
                        Relationship};
use bevy_abm::gestation::{conception, RemainingGestation, update_gestation};
use bevy_abm::config::SimulationParameters;

#[test]
fn test_death_stops_conception() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // High enough for our test
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        min_conception_age: 22.0,
        max_conception_age: 40.0,
        conception_rate: 10.0, // Very high rate to ensure conception attempts
        gestation_duration: 0.1, // Very short gestation for testing
        ..Default::default()
    };
    world.insert_resource(params.clone());
    world.insert_resource(AvailableSeekers::default());

    // Create entities: 1 male, 1 female - both adults, conception-ready age
    let male1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Male }
    )).id();

    let female1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Female }
    )).id();

    // Setup partnership pipeline
    let mut partnership_schedule = Schedule::default();
    partnership_schedule.add_systems((
        start_partner_seeking,
        queue_partner_seekers,
        match_partners,
        resolve_matches
    ));

    // Run partnership formation until it happens
    for _ in 0..10 {
        partnership_schedule.run(&mut world);
        // Check if partnership formed
        if world.get::<Partner>(male1).is_some() && world.get::<Partner>(female1).is_some() {
            break;
        }
    }

    // Verify partnership formed
    assert!(world.get::<Partner>(male1).is_some(), "Male should have Partner component");
    assert!(world.get::<Partner>(female1).is_some(), "Female should have Partner component");

    let relationships = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships, 1, "Should have 1 relationship");

    // Setup conception system only (separate from gestation updates)
    let mut conception_schedule = Schedule::default();
    conception_schedule.add_systems(conception);

    // Run conception until it happens (with very high conception rate, should happen quickly)
    for _ in 0..50 {
        conception_schedule.run(&mut world);
        // Check if conception happened
        let gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
        if gestation_count > 0 {
            break;
        }
    }

    // Verify conception occurred
    let gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
    assert!(gestation_count > 0, "Female should have conceived with high conception rate");

    // Now run gestation updates to let gestation complete (very short duration)
    let mut gestation_schedule = Schedule::default();
    gestation_schedule.add_systems(update_gestation);
    for _ in 0..20 {
        gestation_schedule.run(&mut world);
    }

    // Verify gestation completed
    let gestation_after_birth = world.query::<&RemainingGestation>().iter(&world).count();
    assert_eq!(gestation_after_birth, 0, "Gestation should have completed");

    // Now simulate male death by despawning (which automatically removes Partner component)
    world.despawn(male1);

    // Setup widow detection system
    let mut widow_schedule = Schedule::default();
    widow_schedule.add_systems(detect_widows);

    // Run widow detection
    widow_schedule.run(&mut world);

    // Verify female no longer has Partner component
    assert!(world.get::<Partner>(female1).is_none(), "Female should lose Partner component when partner dies");

    // Verify relationship entity is cleaned up
    let relationships_after = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships_after, 0, "Relationship should be cleaned up after death");

    // Now try conception again - should not work since no Partner component
    let mut conception_only_schedule = Schedule::default();
    conception_only_schedule.add_systems(conception);
    for _ in 0..10 {
        conception_only_schedule.run(&mut world);
        // No break condition needed - we expect this to never succeed
    }

    // Should not create new gestations since female has no partner
    let final_gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
    assert_eq!(final_gestation_count, 0, "No new conception should occur without partner");
}

#[test]
fn test_breakup_stops_conception() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // High enough for our test
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        min_conception_age: 22.0,
        max_conception_age: 40.0,
        conception_rate: 10.0, // Very high rate to ensure conception attempts
        breakup_rate: 100.0, // Very high rate to ensure breakup
        gestation_duration: 0.1, // Very short gestation for testing
        ..Default::default()
    };
    world.insert_resource(params.clone());
    world.insert_resource(AvailableSeekers::default());

    // Create entities: 1 male, 1 female - both adults, conception-ready age
    let male1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Male }
    )).id();

    let female1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Female }
    )).id();

    // Setup partnership pipeline (without breakups initially)
    let mut partnership_schedule = Schedule::default();
    partnership_schedule.add_systems((
        start_partner_seeking,
        queue_partner_seekers,
        match_partners,
        resolve_matches
    ));

    // Run partnership formation until it happens
    for _ in 0..10 {
        partnership_schedule.run(&mut world);
        // Check if partnership formed
        if world.get::<Partner>(male1).is_some() && world.get::<Partner>(female1).is_some() {
            break;
        }
    }

    // Verify partnership formed
    assert!(world.get::<Partner>(male1).is_some(), "Male should have Partner component");
    assert!(world.get::<Partner>(female1).is_some(), "Female should have Partner component");

    let relationships = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships, 1, "Should have 1 relationship");

    // Setup conception system only (separate from gestation updates)
    let mut conception_schedule = Schedule::default();
    conception_schedule.add_systems(conception);

    // Run conception until it happens (with very high conception rate, should happen quickly)
    for _ in 0..50 {
        conception_schedule.run(&mut world);
        // Check if conception happened
        let gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
        if gestation_count > 0 {
            break;
        }
    }

    // Verify conception occurred
    let gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
    assert!(gestation_count > 0, "Female should have conceived with high conception rate");

    // Now run gestation updates to let gestation complete (very short duration)
    let mut gestation_schedule = Schedule::default();
    gestation_schedule.add_systems(update_gestation);
    for _ in 0..20 {
        gestation_schedule.run(&mut world);
    }

    // Verify gestation completed
    let gestation_after_birth = world.query::<&RemainingGestation>().iter(&world).count();
    assert_eq!(gestation_after_birth, 0, "Gestation should have completed");

    // Now add breakup system and trigger breakup (with very high breakup rate, should happen quickly)
    let mut breakup_schedule = Schedule::default();
    breakup_schedule.add_systems(random_breakups);

    for _ in 0..10 {
        breakup_schedule.run(&mut world);
        // Check if breakup happened
        if world.get::<Partner>(male1).is_none() || world.get::<Partner>(female1).is_none() {
            break;
        }
    }

    // Verify both entities no longer have Partner components
    assert!(world.get::<Partner>(male1).is_none(), "Male should lose Partner component after breakup");
    assert!(world.get::<Partner>(female1).is_none(), "Female should lose Partner component after breakup");

    // Verify relationship entity is cleaned up
    let relationships_after = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships_after, 0, "Relationship should be cleaned up after breakup");

    // Now try conception again - should not work since no Partner components
    let mut conception_only_schedule = Schedule::default();
    conception_only_schedule.add_systems(conception);
    for _ in 0..10 {
        conception_only_schedule.run(&mut world);
    }

    // Should not create new gestations since entities have no partners
    let final_gestation_count = world.query::<&RemainingGestation>().iter(&world).count();
    assert_eq!(final_gestation_count, 0, "No new conception should occur without partners");
}