use rltk::{field_of_view, Point};
use specs::prelude::*;

use crate::{
    components::{Hidden, Name, Player, Position, Viewshed},
    gamelog::GameLog,
    map::Map,
};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
    );

    fn run(
        &mut self,
        (mut map, entities, mut viewshed, pos, player, names, mut hidden, mut rng, mut gamelog): Self::SystemData,
    ) {
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                let player_ent = player.get(ent);
                if let Some(_) = player_ent {
                    for tile_visible in map.visible_tiles.iter_mut() {
                        *tile_visible = false;
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        for tile_content in map.tile_content[idx].iter() {
                            if let Some(_) = hidden.get(*tile_content) {
                                if rng.roll_dice(1, 24) == 1 {
                                    if let Some(name) = names.get(*tile_content) {
                                        gamelog
                                            .entries
                                            .push(format!("You spotted a {}.", &name.name));
                                    }
                                    hidden.remove(*tile_content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
