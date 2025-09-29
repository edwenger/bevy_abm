use std::cmp::min;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use crate::individual::{
    Individual, Demog, Adult, Sex
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
            (
                queue_partner_seekers,
                match_partners,
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
pub struct Partner(Entity);

#[derive(Component)]
pub struct Relationship;

#[derive(Component)]
pub struct Partners {
    pub e1: Entity,
    pub e2: Entity
}

#[derive(Default, Resource)]
pub struct AvailableSeekers {
    females: Vec<Entity>,
    males: Vec<Entity>
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
    query: Query<Entity, (Without<PartnerSeeking>, Without<Partner>, With<Adult>)>
) {
    for e in query.iter() {
        eprintln!("Entity {:?} beginning partner-seeking", e);
        commands.entity(e).insert(PartnerSeeking);            
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
) {
    // Dummy example matching scheme: FIFO
    
    let it1 = cache.females.iter(); // drain() removes from vector as iterated (with unequal length kept after zip()
    let it2 = cache.males.iter();  // ugh: can't borrow mutable cache more than once at a time (w/o extra Rust skills)

    for (e1, e2) in it1.zip(it2) {
        commands
            .spawn(Relationship)
            .insert(Partners{
                e1: *e1,
                e2: *e2
            });
        eprintln!("New relationship between {:?} and {:?}", e1, e2);
    }

    let min_len = min(cache.females.len(), cache.males.len());
    // eprintln!("Minimum queue of partner seekers = {}", min_len);

    cache.males = cache.males.split_off(min_len);  // leave unmatched partner-seekers for next round
    cache.females = cache.females.split_off(min_len);
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

pub fn detect_widows(
    mut commands: Commands,
    mut removals: RemovedComponents<Partner>,
    query: Query<&Partner>
) {
    for entity in removals.read() {
        eprintln!("{:?} detected removal of Partner component", entity);
        if let Ok(partner) = query.get(entity) {
            eprintln!("{:?} died + notified their partner {:?}", entity, partner.0);
            commands.entity(partner.0).remove::<Partner>();  // remove Partner component from entities whose partner has removed theirs (e.g. by despawning)
        } else {
            eprintln!("No such entity in query for Partner component?");  // TODO: this line prints because the component isn't there!
        }
    }
}