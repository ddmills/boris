use bevy::ecs::{
    event::{Event, EventReader},
    system::ResMut,
};

use crate::{terrain, BlockType, Terrain};

pub struct Tree {
    canopy: Vec<[i32; 3]>,
    trunk: Vec<[i32; 3]>,
}

pub struct TreeSettings {
    pub height: u32,
    pub canopy_radius: u32,
}

#[derive(Event)]
pub struct SpawnTreeEvent {
    pub position: [u32; 3],
    pub settings: TreeSettings,
}

pub fn build_tree(settings: &TreeSettings) -> Tree {
    // let trunk = (0..=(settings.height as i32))
    //     .map(|i| [0, i, 0])
    //     .collect::<Vec<_>>();

    let height = settings.height as i32;
    let can_radius = settings.canopy_radius as i32;

    let mut trunk = vec![];
    let mut canopy = vec![[0, height, 0]];
    let canopy_end = (height / 3) * 2;

    for h in 0..height {
        if h >= canopy_end {
            for x in -can_radius..=can_radius {
                for y in -can_radius..=can_radius {
                    canopy.push([x, h, y]);
                }
            }
        }

        trunk.push([0, h, 0]);
    }

    Tree { trunk, canopy }
}

pub fn on_spawn_tree(mut ev_spawn_tree: EventReader<SpawnTreeEvent>, mut terrain: ResMut<Terrain>) {
    for ev in ev_spawn_tree.read() {
        let tree = build_tree(&ev.settings);
        let pos = ev.position;

        for part in tree.trunk.iter() {
            let x = pos[0] as i32 + part[0];
            let y = pos[1] as i32 + part[1];
            let z = pos[2] as i32 + part[2];

            let current = terrain.get_block_i32(x, y, z);

            if current.is_oob() {
                continue;
            }

            // TODO note: blueprints are also considered "empty"
            if current.is_empty() {
                terrain.set_block_type(x as u32, y as u32, z as u32, BlockType::TREE_TRUNK);
            }
        }

        for part in tree.canopy.iter() {
            let x = pos[0] as i32 + part[0];
            let y = pos[1] as i32 + part[1];
            let z = pos[2] as i32 + part[2];

            let current = terrain.get_block_i32(x, y, z);

            if current.is_oob() {
                continue;
            }

            // TODO note: blueprints are also considered "empty"
            if current.is_empty() {
                terrain.set_block_type(x as u32, y as u32, z as u32, BlockType::LEAVES);
            }
        }
    }
}
