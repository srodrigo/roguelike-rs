use specs::saveload::{DeserializeComponents, SerializeComponents};
use specs::{error::NoError, saveload::SimpleMarkerAllocator};
use std::{
    fs::{self, File},
    path::Path,
};

use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{components::*, map::Map, map::MAP_SIZE};

static SAVE_GAME_FILENAME: &str = "./savegame.json";

macro_rules! serialize_individually {
    ($world:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &($world.read_storage::<$type>(),),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($world:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut (&mut $world.write_storage::<$type>(),),
            &$data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(world: &mut World) {
    let map_copy = world.get_mut::<super::map::Map>().unwrap().clone();
    let save_helper = world
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    {
        let data = (
            world.entities(),
            world.read_storage::<SimpleMarker<SerializeMe>>(),
        );
        let writer = File::create(SAVE_GAME_FILENAME).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            world,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SuffersDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood
        );
    }

    world.delete_entity(save_helper).expect("Crash on cleanup");
}

pub fn load_game(world: &mut World) {
    {
        let mut to_delete = Vec::new();
        for entity in world.entities().join() {
            to_delete.push(entity);
        }
        for entity in to_delete.iter() {
            world.delete_entity(*entity).expect("Deletion failed");
        }
    }

    let game_data = fs::read_to_string(SAVE_GAME_FILENAME).unwrap();
    let mut deserialized_game = serde_json::Deserializer::from_str(&game_data);

    {
        let mut data = (
            &mut world.entities(),
            &mut world.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut world.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );
        deserialize_individually!(
            world,
            deserialized_game,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SuffersDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickUpItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToRemoveItem,
            ParticleLifetime,
            HungerClock,
            ProvidesFood
        );
    }

    let mut helper_entity: Option<Entity> = None;
    {
        let entities = world.entities();
        let helper = world.read_storage::<SerializationHelper>();
        let player = world.read_storage::<Player>();
        let position = world.read_storage::<Position>();
        for (entity, help) in (&entities, &helper).join() {
            let mut map = world.write_resource::<Map>();
            *map = help.map.clone();
            map.tile_content = vec![Vec::new(); MAP_SIZE];
            helper_entity = Some(entity);
        }
        for (entity, _player, pos) in (&entities, &player, &position).join() {
            let mut point = world.write_resource::<rltk::Point>();
            *point = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = world.write_resource::<Entity>();
            *player_resource = entity;
        }
    }

    world
        .delete_entity(helper_entity.unwrap())
        .expect("Unable to delete helper");
}

pub fn delete_saved_game() {
    if is_game_saved() {
        std::fs::remove_file(SAVE_GAME_FILENAME).expect("Unable to delete file");
    }
}

pub fn is_game_saved() -> bool {
    Path::new(SAVE_GAME_FILENAME).exists()
}
