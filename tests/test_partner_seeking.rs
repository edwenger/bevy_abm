#[macro_use]
extern crate approx;

use bevy::prelude::*;

use bevy_abm::individual::{Individual, Demog, Sex, Adult, update_age};
use bevy_abm::partner::{PartnerSeeking, start_partner_seeking};

#[test]
fn did_start_partner_seeking() {

    // Setup world
    let mut world = World::default();

    // Setup test entities
    let individual_id = world.spawn((Individual, Demog{ age: 19.5, sex: Sex::Male})).id();

    // Check before state
    assert!(world.get::<Individual>(individual_id).is_some());
    assert!(relative_eq!(world.get::<Demog>(individual_id).unwrap().age, 19.5, epsilon = f32::EPSILON));
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
    assert!(relative_eq!(world.get::<Demog>(individual_id).unwrap().age, 20.5, epsilon = f32::EPSILON));
    assert!(world.get::<Adult>(individual_id).is_some());
    assert!(world.get::<PartnerSeeking>(individual_id).is_some());
}