use crate::{
    components::{CombatStats, Name, Player, SufferDamage},
    gamelog::GameLog,
};
use rltk::console;
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(world: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = world.read_storage::<CombatStats>();
        let players = world.read_storage::<Player>();
        let names = world.read_storage::<Name>();
        let entities = world.entities();
        let mut log = world.write_resource::<GameLog>();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                match players.get(entity) {
                    None => {
                        if let Some(victim_name) = names.get(entity) {
                            log.entries.push(format!("{} is dead", &&victim_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => log.entries.push("You are dead".to_string()),
                }
            }
        }
    }

    for victim in dead {
        world.delete_entity(victim).expect("Unable to delete");
    }
}
