#[macro_use]
extern crate approx;

use bevy::prelude::*;
use bevy::app::Events;

use bevy_abm::individual::*;

#[test]
fn did_start_partner_seeking() {

    // Setup world
    let mut world = World::default();
    world.insert_resource(Events::<DeathEvent>::default());
    world.insert_resource(Events::<BecomeAdultEvent>::default());

    // Setup stage with aging system
    let mut update_stage = SystemStage::parallel();
    update_stage.add_system(update_age.label("aging"));
    update_stage.add_system(start_partner_seeking.after("aging"));

    // Setup test entities
    let individual_id = world.spawn().insert(Individual).insert(Demog { age: 15.0, sex: Sex::Male }).id();

    // Run systems
    for _tstep in 0..12 {
        update_stage.run(&mut world);
    }

    // Check resulting changes
    assert!(world.get::<Individual>(individual_id).is_some());
    relative_eq!(world.get::<Demog>(individual_id).unwrap().age, 16.0, epsilon = f32::EPSILON);
    // assert!(world.get::<PartnerSeeking>(individual_id).is_some());

}