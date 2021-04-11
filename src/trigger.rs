use specs::prelude::*;

use crate::{
    components::{
        EntityMoved, EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation,
        SuffersDamage,
    },
    gamelog::GameLog,
    map::Map,
    particles::ParticlesBuilder,
};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SuffersDamage>,
        ReadStorage<'a, SingleActivation>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, ParticlesBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            inflicts_damage,
            mut suffers_damage,
            single_activations,
            mut hidden,
            names,
            entities,
            mut gamelog,
            mut particles_builder,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    match entry_trigger.get(*entity_id) {
                        None => {}
                        Some(_trigger) => {
                            if let Some(name) = names.get(*entity_id) {
                                gamelog.entries.push(format!("{} triggers!", &name.name));
                            }

                            if let Some(damage) = inflicts_damage.get(*entity_id) {
                                particles_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                                SuffersDamage::new_damage(
                                    &mut suffers_damage,
                                    entity,
                                    damage.damage,
                                );
                            }

                            if let Some(_) = single_activations.get(*entity_id) {
                                remove_entities.push(*entity_id);
                            }

                            hidden.remove(*entity_id);
                        }
                    }
                }
            }
        }

        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        entity_moved.clear();
    }
}
