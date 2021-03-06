use map::Map;
use rltk::Point;
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

mod components;
use components::*;

mod map;

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
mod trigger;
mod visibility;

mod maps;
mod random_table;

const SHOW_MAPGEN_VISUALIZER: bool = true;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut game_state = State {
        world: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_history: Vec::new(),
        mapgen_index: 0,
        mapgen_timer: 0.0,
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
    game_state.world.register::<Hidden>();
    game_state.world.register::<EntryTrigger>();
    game_state.world.register::<EntityMoved>();
    game_state.world.register::<SingleActivation>();
    game_state.world.register::<SimpleMarker<SerializeMe>>();
    game_state.world.register::<SerializationHelper>();

    game_state.world.insert(RunState::MapGeneration {});
    game_state
        .world
        .insert(SimpleMarkerAllocator::<SerializeMe>::new());

    game_state.world.insert(particles::ParticlesBuilder::new());

    game_state.world.insert(Map::new(1));
    game_state.world.insert(Point::new(0, 0));
    game_state.world.insert(RandomNumberGenerator::new());

    let player = spawner::player(&mut game_state.world, 0, 0);
    game_state.world.insert(player);

    game_state.world.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    game_state.generate_world_map(1);

    rltk::main_loop(context, game_state)
}
