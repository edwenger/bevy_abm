use std::cmp::min;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use crate::individual::{
    Individual, Demog, Adult, Elder, Sex
};
use crate::config::SimulationParameters;

pub struct PartnerPlugin;

impl Plugin for PartnerPlugin {
    fn build(&self, app: &mut App) {
        app

        //-- PARTNERS
        .init_resource::<AvailableSeekers>()
        .add_systems(Update, (
            start_partner_seeking,
            stop_elder_partner_seeking,
            (
                queue_partner_seekers,
                match_partners,
                random_breakups,
            ).run_if(on_timer(Duration::from_secs_f32(SEEKING_TIMESTEP))),
            resolve_matches,
        ))
        .add_systems(PostUpdate, detect_widows);
    }
}

//-- PARTNERS
const SEEKING_TIMESTEP: f32 = 1.0/4.0;  // N.B. slower for testing via printout + visualization
// PARTNER_SEEKING_AGE now comes from SimulationParameters

// ------ PARTNER ------

#[derive(Component)]
pub struct PartnerSeeking;

#[derive(Component)]
pub struct Partner(pub Entity);

#[derive(Component)]
pub struct Relationship;

#[derive(Component)]
pub struct Partners {
    pub e1: Entity,
    pub e2: Entity
}

#[derive(Default, Resource)]
pub struct AvailableSeekers {
    pub females: Vec<Entity>,
    pub males: Vec<Entity>
}
impl AvailableSeekers {
    fn add_seeker(&mut self, e: Entity, sex: Sex) {
        match sex {
            Sex::Female => self.females.push(e),
            _ => self.males.push(e),
        }
    }
    // fn clear(&mut self) {
    //     self.females.clear();
    //     self.males.clear();
    // }

}

pub fn start_partner_seeking(
    mut commands: Commands,
    query: Query<Entity, (Without<PartnerSeeking>, Without<Partner>, With<Adult>, Without<Elder>)>
) {
    for e in query.iter() {
        eprintln!("Entity {:?} beginning partner-seeking", e);
        commands.entity(e).insert(PartnerSeeking);            
    }
}

pub fn stop_elder_partner_seeking(
    mut commands: Commands,
    query: Query<Entity, (With<PartnerSeeking>, Added<Elder>)>
) {
    for e in query.iter() {
        eprintln!("Entity {:?} stopped partner-seeking (became elder)", e);
        commands.entity(e).remove::<PartnerSeeking>();
    }
}

pub fn queue_partner_seekers(
    mut cache: ResMut<AvailableSeekers>,
    query: Query<(Entity, &Demog), 
                //  With<PartnerSeeking>
                 Added<PartnerSeeking>
                 >
) {
    // cache.clear();  // TODO: clear and repopulate queues periodically using With<> rather than Added<> query?

    for (e, d) in query.iter() {
        cache.add_seeker(e, d.sex);
    }
}

pub fn match_partners(
    mut cache: ResMut<AvailableSeekers>,
    mut commands: Commands,
    seeker_query: Query<&Individual, (With<PartnerSeeking>, Without<Partner>, Without<Elder>)>
) {
    // Filter out invalid entities (dead/elder) and match valid ones
    let valid_females: Vec<Entity> = cache.females.iter()
        .filter(|&&e| seeker_query.get(e).is_ok())
        .copied()
        .collect();

    let valid_males: Vec<Entity> = cache.males.iter()
        .filter(|&&e| seeker_query.get(e).is_ok())
        .copied()
        .collect();

    // Match valid seekers using FIFO
    let min_len = min(valid_females.len(), valid_males.len());
    for i in 0..min_len {
        let e1 = valid_females[i];
        let e2 = valid_males[i];
        commands
            .spawn(Relationship)
            .insert(Partners{
                e1: e1,
                e2: e2
            });
        eprintln!("New relationship between {:?} and {:?}", e1, e2);
    }

    // Update cache: keep only unmatched valid entities
    cache.females = valid_females.into_iter().skip(min_len).collect();
    cache.males = valid_males.into_iter().skip(min_len).collect();
}

pub fn resolve_matches(
    mut commands: Commands,
    rel_query: Query<(Entity, &Partners), Added<Relationship>>,
    ind_query: Query<&Individual, (Without<Partner>, With<PartnerSeeking>)>
) {
    for (rel_entity, partners) in rel_query.iter() {
        if let Ok(_ind1) = ind_query.get(partners.e1) {
            if let Ok(_ind2) = ind_query.get(partners.e2) {
                commands.entity(partners.e1).insert(Partner(partners.e2)).remove::<PartnerSeeking>();
                commands.entity(partners.e2).insert(Partner(partners.e1)).remove::<PartnerSeeking>();
            } else {
                eprintln!("{:?} has already despawned", partners.e2);
                commands.entity(rel_entity).despawn();
                commands.entity(partners.e1).remove::<PartnerSeeking>();  // will be added back to try again at finding a partner
            }
        } else {
            eprintln!("{:?} has already despawned", partners.e1);
            commands.entity(rel_entity).despawn();
            commands.entity(partners.e2).remove::<PartnerSeeking>();  // will be added back to try again at finding a partner
        }
    }
}

pub fn random_breakups(
    mut commands: Commands,
    rel_query: Query<(Entity, &Partners), With<Relationship>>,
    params: Res<SimulationParameters>
) {
    use rand::prelude::random;

    // Convert breakup_rate (per year) to probability per SEEKING_TIMESTEP (quarterly check)
    // Using same exponential conversion as conception: prob = 1 - exp(-timestep * rate)
    for (rel_entity, partners) in rel_query.iter() {
        let breakup_prob = 1.0 - (-SEEKING_TIMESTEP * params.breakup_rate).exp();
        if random::<f32>() < breakup_prob {
            eprintln!("Relationship between {:?} and {:?} ended in breakup", partners.e1, partners.e2);

            // Remove Partner components from both entities (they'll re-enter partner seeking)
            commands.entity(partners.e1).remove::<Partner>();
            commands.entity(partners.e2).remove::<Partner>();

            // Despawn the relationship entity
            commands.entity(rel_entity).despawn();
        }
    }
}

pub fn detect_widows(
    mut commands: Commands,
    mut removals: RemovedComponents<Partner>,
    rel_query: Query<(Entity, &Partners), With<Relationship>>
) {
    for dead_entity in removals.read() {
        eprintln!("{:?} detected removal of Partner component", dead_entity);

        // Find the relationship entity that contains this dead entity
        for (rel_entity, partners) in rel_query.iter() {
            if partners.e1 == dead_entity {
                eprintln!("{:?} died + notified their partner {:?}", dead_entity, partners.e2);
                commands.entity(partners.e2).remove::<Partner>();
                commands.entity(rel_entity).despawn(); // clean up the relationship entity
                break;
            } else if partners.e2 == dead_entity {
                eprintln!("{:?} died + notified their partner {:?}", dead_entity, partners.e1);
                commands.entity(partners.e1).remove::<Partner>();
                commands.entity(rel_entity).despawn(); // clean up the relationship entity
                break;
            }
        }
    }
}