use rltk::Point;
use specs::prelude::*;

use crate::{
    components::{Confusion, Monster, Position, Viewshed, WantsToMelee},
    map::Map,
    particles::ParticlesBuilder,
    RunState,
};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticlesBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            run_state,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
            mut particles_builder,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let is_confused = confused.get_mut(entity);
            match is_confused {
                Some(monster_confused) => {
                    monster_confused.turns -= 1;
                    if monster_confused.turns < 1 {
                        confused.remove(entity);
                    }

                    particles_builder.request(
                        pos.x,
                        pos.y,
                        rltk::RGB::named(rltk::MAGENTA),
                        rltk::RGB::named(rltk::BLACK),
                        rltk::to_cp437('?'),
                        200.0,
                    );
                }
                None => {
                    let distance = rltk::DistanceAlg::Pythagoras
                        .distance2d(Point::new(pos.x, pos.y), *player_pos);
                    if distance < 1.5 {
                        wants_to_melee
                            .insert(
                                entity,
                                WantsToMelee {
                                    target: *player_entity,
                                },
                            )
                            .expect("Unable to insert attack");
                    } else if viewshed.visible_tiles.contains(&*player_pos) {
                        let path = rltk::a_star_search(
                            map.xy_idx(pos.x, pos.y),
                            map.xy_idx(player_pos.x, player_pos.y),
                            &mut *map,
                        );
                        if path.success && path.steps.len() > 1 {
                            let mut idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = false;
                            pos.x = path.steps[1] as i32 % map.width;
                            pos.y = path.steps[1] as i32 / map.width;
                            idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = true;
                            viewshed.dirty = true;
                        }
                    }
                }
            }
        }
    }
}
