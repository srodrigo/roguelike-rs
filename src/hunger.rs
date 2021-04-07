use specs::prelude::*;

use crate::{
    components::{HungerClock, SuffersDamage},
    gamelog::GameLog,
    state::RunState,
};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SuffersDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage, mut log) =
            data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false,
            }

            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        crate::components::HungerState::WellFed => {
                            clock.state = crate::components::HungerState::Normal;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are no longer well fed.".to_string());
                            }
                        }
                        crate::components::HungerState::Normal => {
                            clock.state = crate::components::HungerState::Hungry;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are hungry".to_string());
                            }
                        }
                        crate::components::HungerState::Hungry => {
                            clock.state = crate::components::HungerState::Starving;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are starving!".to_string());
                            }
                        }
                        crate::components::HungerState::Starving => {
                            if entity == *player_entity {
                                log.entries.push("Your hunger pangs are getting painful! You suffer 1 hp damage.".to_string());
                            }
                            SuffersDamage::new_damage(&mut inflict_damage, entity, 1);
                        }
                    }
                }
            }
        }
    }
}
