use rltk::Point;
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
use components::*;

mod map;
use map::*;

mod player;

mod rect;

mod gui;

mod state;
use state::{RunState, State};

mod gamelog;
use gamelog::GameLog;

mod spawner;

mod damage;
mod hunger;
mod inventory;
mod map_indexing;
mod melee_combat;
mod monster_ai;
mod particles;
mod saveload;
mod visibility;

mod random_table;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut game_state = State {
        world: World::new(),
    };
    game_state.world.register::<Position>();
    game_state.world.register::<Renderable>();
    game_state.world.register::<Player>();
    game_state.world.register::<Monster>();
    game_state.world.register::<Name>();
    game_state.world.register::<Viewshed>();
    game_state.world.register::<BlocksTile>();
    game_state.world.register::<CombatStats>();
    game_state.world.register::<WantsToMelee>();
    game_state.world.register::<SuffersDamage>();
    game_state.world.register::<Item>();
    game_state.world.register::<ProvidesHealing>();
    game_state.world.register::<InBackpack>();
    game_state.world.register::<WantsToPickUpItem>();
    game_state.world.register::<WantsToDropItem>();
    game_state.world.register::<WantsToUseItem>();
    game_state.world.register::<Consumable>();
    game_state.world.register::<Ranged>();
    game_state.world.register::<InflictsDamage>();
    game_state.world.register::<AreaOfEffect>();
    game_state.world.register::<Confusion>();
    game_state.world.register::<Equippable>();
    game_state.world.register::<Equipped>();
    game_state.world.register::<MeleePowerBonus>();
    game_state.world.register::<DefenseBonus>();
    game_state.world.register::<WantsToRemoveItem>();
    game_state.world.register::<ParticleLifetime>();
    game_state.world.register::<HungerClock>();
    game_state.world.register::<ProvidesFood>();
    game_state.world.register::<MagicMapper>();
    game_state.world.register::<SimpleMarker<SerializeMe>>();
    game_state.world.register::<SerializationHelper>();

    game_state.world.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    game_state
        .world
        .insert(SimpleMarkerAllocator::<SerializeMe>::new());

    game_state.world.insert(particles::ParticlesBuilder::new());

    let map = Map::new_map_rooms_and_corridors(1);

    game_state.world.insert(RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut game_state.world, room, 1);
    }

    let (player_x, player_y) = map.rooms[0].center();
    game_state.world.insert(Point::new(player_x, player_y));

    game_state.world.insert(map);

    let player = spawner::player(&mut game_state.world, player_x, player_y);
    game_state.world.insert(player);

    game_state.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    rltk::main_loop(context, game_state)
}
