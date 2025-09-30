#[macro_use]
extern crate approx;

use bevy::prelude::*;

use bevy_abm::individual::{Individual, Demog, Sex, Adult, Elder, update_age};
use bevy_abm::partner::{PartnerSeeking, start_partner_seeking, stop_elder_partner_seeking,
                        queue_partner_seekers, match_partners, resolve_matches,
                        AvailableSeekers, Partner, Relationship, Partners,
                        detect_widows, BreakupEvent};
use bevy_abm::config::SimulationParameters;

#[test]
fn did_start_partner_seeking() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // High enough to not interfere with Adult transition
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        ..Default::default()
    };
    world.insert_resource(params.clone());

    // Setup test entity just below min partner seeking age
    let start_age = params.min_partner_seeking_age - 0.5;
    let individual_id = world.spawn((Individual, Demog{ age: start_age, sex: Sex::Male})).id();

    // Check before state
    assert!(world.get::<Individual>(individual_id).is_some());
    assert!(relative_eq!(world.get::<Demog>(individual_id).unwrap().age, start_age, epsilon = f32::EPSILON));
    assert!(world.get::<Adult>(individual_id).is_none());
    assert!(world.get::<PartnerSeeking>(individual_id).is_none());

    // Setup and run systems
    let mut schedule = Schedule::default();
    schedule.add_systems((update_age, start_partner_seeking));

    // Run systems multiple times to simulate aging
    for _tstep in 0..12 {  // dependent on AGING_TIMESTEP
        schedule.run(&mut world);
    }

    // Check resulting changes
    let expected_age = start_age + 1.0; // 12 timesteps * (1/12) per timestep
    let actual_age = world.get::<Demog>(individual_id).unwrap().age;
    // println!("[START TEST] Start: {}, Expected: {}, Actual: {}", start_age, expected_age, actual_age);
    assert!(relative_eq!(actual_age, expected_age, epsilon = 0.001));
    assert!(world.get::<Adult>(individual_id).is_some());
    assert!(world.get::<PartnerSeeking>(individual_id).is_some());
}

#[test]
fn did_stop_partner_seeking() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // High enough to not interfere with Elder transition
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        ..Default::default()
    };
    world.insert_resource(params.clone());

    // Setup test entity just below max partner seeking age with PartnerSeeking
    let start_age = params.max_partner_seeking_age - 0.5;
    let individual_id = world.spawn((
        Individual,
        Adult,
        PartnerSeeking,
        Demog { age: start_age, sex: Sex::Female }
    )).id();

    // Check before state
    assert!(world.get::<Individual>(individual_id).is_some());
    assert!(relative_eq!(world.get::<Demog>(individual_id).unwrap().age, start_age, epsilon = f32::EPSILON));
    assert!(world.get::<Adult>(individual_id).is_some());
    assert!(world.get::<PartnerSeeking>(individual_id).is_some());
    assert!(world.get::<Elder>(individual_id).is_none());

    // Setup and run systems
    let mut schedule = Schedule::default();
    schedule.add_systems((update_age, stop_elder_partner_seeking));

    // Run systems multiple times to age past max_partner_seeking_age
    for _tstep in 0..12 {  // dependent on AGING_TIMESTEP
        schedule.run(&mut world);
    }

    // Check resulting changes
    let expected_age = start_age + 1.0; // 12 timesteps * (1/12) per timestep
    let actual_age = world.get::<Demog>(individual_id).unwrap().age;
    // println!("[STOP TEST] Start: {}, Expected: {}, Actual: {}", start_age, expected_age, actual_age);
    assert!(relative_eq!(actual_age, expected_age, epsilon = 0.001));
    assert!(world.get::<Adult>(individual_id).is_some());
    assert!(world.get::<Elder>(individual_id).is_some());
    assert!(world.get::<PartnerSeeking>(individual_id).is_none()); // Should be removed when became Elder
}

#[test]
fn test_partner_matching_with_uneven_ratios() {

    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // High enough to not interfere with matching
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        ..Default::default()
    };
    world.insert_resource(params.clone());
    world.insert_resource(AvailableSeekers::default());

    // Create entities: 2 males, 1 female - all adults but not yet seeking partners
    let male1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 25.0, sex: Sex::Male }
    )).id();

    let male2 = world.spawn((
        Individual,
        Adult,
        Demog { age: 26.0, sex: Sex::Male }
    )).id();

    let female1 = world.spawn((
        Individual,
        Adult,
        Demog { age: 24.0, sex: Sex::Female }
    )).id();

    // Check initial state - none should be partner seeking yet
    assert!(world.get::<PartnerSeeking>(male1).is_none());
    assert!(world.get::<PartnerSeeking>(male2).is_none());
    assert!(world.get::<PartnerSeeking>(female1).is_none());
    assert!(world.get::<Partner>(male1).is_none());
    assert!(world.get::<Partner>(male2).is_none());
    assert!(world.get::<Partner>(female1).is_none());

    // Setup and run the full partner matching pipeline
    let mut schedule = Schedule::default();
    schedule.add_systems((
        start_partner_seeking,
        queue_partner_seekers,
        match_partners,
        resolve_matches
    ));

    // Run the matching pipeline until partnerships are resolved
    for _ in 0..10 {
        schedule.run(&mut world);
        // Check if we have the expected number of partners (2 out of 3 entities)
        let partner_count = world.query::<&Partner>().iter(&world).count();
        if partner_count == 2 {
            break;
        }
    }

    println!("After pipeline runs:");
    let relationships_after = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    let partners_after = world.query::<&Partner>().iter(&world).count();
    let seeking_after = world.query::<&PartnerSeeking>().iter(&world).count();
    println!("Relationships: {}, Partners: {}, Seeking: {}", relationships_after, partners_after, seeking_after);

    // Check results: should have 1 relationship, 1 unmatched male
    let relationships: Vec<Entity> = world.query_filtered::<Entity, With<Relationship>>().iter(&world).collect();
    assert_eq!(relationships.len(), 1, "Should have exactly 1 relationship");

    // Check that exactly 2 entities have Partner components (the matched pair)
    let partner_count = world.query::<&Partner>().iter(&world).count();
    assert_eq!(partner_count, 2, "Should have exactly 2 entities with Partner component");

    // Check that exactly 1 entity still has PartnerSeeking (the unmatched male)
    let seeking_count = world.query::<&PartnerSeeking>().iter(&world).count();
    assert_eq!(seeking_count, 1, "Should have exactly 1 entity still seeking partner");

    // Check resource queue state - should have 1 unmatched male
    let seekers = world.resource::<AvailableSeekers>();
    assert_eq!(seekers.females.len(), 0, "Should have 0 females in queue");
    assert_eq!(seekers.males.len(), 1, "Should have 1 male in queue");

    // Verify the relationship contains the expected entities
    let mut relationship_query = world.query::<&Partners>();
    let partners = relationship_query.single(&world);

    // One of the partners should be the female, the other should be one of the males
    let partner_entities = [partners.e1, partners.e2];
    assert!(partner_entities.contains(&female1), "Female should be in the relationship");

    let matched_male = if partner_entities[0] == female1 {
        partner_entities[1]
    } else {
        partner_entities[0]
    };
    assert!(matched_male == male1 || matched_male == male2, "One male should be matched");

    let unmatched_male = if matched_male == male1 { male2 } else { male1 };
    assert!(world.get::<PartnerSeeking>(unmatched_male).is_some(), "Unmatched male should still be seeking");
    assert!(world.get::<Partner>(unmatched_male).is_none(), "Unmatched male should not have Partner component");
}

#[test]
fn test_simultaneous_partner_death() {
    // Setup world with resources
    let mut world = World::default();
    let params = SimulationParameters {
        death_age: 60.0, // We'll change this later to trigger deaths
        min_partner_seeking_age: 20.0,
        max_partner_seeking_age: 50.0,
        min_conception_age: 22.0,
        max_conception_age: 40.0,
        conception_rate: 0.0, // Not testing conception
        gestation_duration: 40.0 / 52.0,
        breakup_rate: 0.0, // No breakups for this test
        spawn_individual_age: 18.0,
    };
    world.insert_resource(params);
    world.insert_resource(AvailableSeekers::default());
    world.insert_resource(Events::<BreakupEvent>::default());
    world.insert_resource(Time::<()>::default());

    // Create male and female with ages in partner-seeking range but above future death age
    let male1 = world.spawn((
        Individual,
        Demog { age: 35.0, sex: Sex::Male }, // In partner seeking range (20-50)
        Adult,
        PartnerSeeking,
    )).id();

    let female1 = world.spawn((
        Individual,
        Demog { age: 34.0, sex: Sex::Female }, // In partner seeking range (20-50)
        Adult,
        PartnerSeeking,
    )).id();

    // Setup partnership formation system
    let mut partnership_schedule = Schedule::default();
    partnership_schedule.add_systems((
        queue_partner_seekers,
        match_partners,
        resolve_matches,
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

    let relationships_before = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships_before, 1, "Should have 1 relationship");

    // Simulate both partners dying simultaneously by manually despawning them
    // This mimics what happens when update_age runs with a lowered death_age (like moving UI slider down)
    world.despawn(male1);
    world.despawn(female1);

    // Both despawns should have removed Partner components, triggering RemovedComponents events

    // Setup widow detection system - this is where the panic should occur
    let mut widow_schedule = Schedule::default();
    widow_schedule.add_systems(detect_widows);

    // This should NOT panic - it should handle duplicate relationship cleanup gracefully
    widow_schedule.run(&mut world);

    // Verify relationship entity is cleaned up (should only happen once, not cause panic)
    let relationships_after = world.query_filtered::<Entity, With<Relationship>>().iter(&world).count();
    assert_eq!(relationships_after, 0, "Relationship should be cleaned up after simultaneous deaths");
}