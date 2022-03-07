#[macro_use]
extern crate approx;

use bevy::prelude::*;

use bevy_abm::individual::*;

#[test]
fn did_start_partner_seeking() {

    // Setup world
    let mut world = World::default();

    // Setup stage with aging system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(update_age.label("aging"));
    update_stage.add_system(start_partner_seeking.after("aging"));

    // Setup test entities
    let individual_id = world.spawn().insert(Individual).insert(Demog{ age: 19.5, sex: Sex::Male}).id();

    // Check before state
    assert!(world.get::<Individual>(individual_id).is_some());
    relative_eq!(world.get::<Demog>(individual_id).unwrap().age, 19.5, epsilon = f32::EPSILON);
    assert!(world.get::<Adult>(individual_id).is_none());
    assert!(world.get::<PartnerSeeking>(individual_id).is_none());

    // Run systems
    for _tstep in 0..12 {  // dependent on AGING_TIMESTEP
        update_stage.run(&mut world);
    }

    // Check resulting changes
    relative_eq!(world.get::<Demog>(individual_id).unwrap().age, 20.5, epsilon = f32::EPSILON);
    assert!(world.get::<Adult>(individual_id).is_some());
    assert!(world.get::<PartnerSeeking>(individual_id).is_some());
}