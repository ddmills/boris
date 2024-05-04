use bevy::{
    core::Name,
    ecs::{
        component::Component,
        event::{Event, EventReader},
        system::{Commands, ResMut},
    },
};

use crate::{BlockType, Terrain};

#[derive(Component)]
pub struct Tree {
    pub canopy: Vec<[u32; 3]>,
    pub trunk: Vec<[u32; 3]>,
}

pub struct TreeTemplate {
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

pub fn build_tree(settings: &TreeSettings) -> TreeTemplate {
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

    TreeTemplate { trunk, canopy }
}

pub fn on_spawn_tree(
    mut cmd: Commands,
    mut ev_spawn_tree: EventReader<SpawnTreeEvent>,
    mut terrain: ResMut<Terrain>,
) {
    for ev in ev_spawn_tree.read() {
        let template = build_tree(&ev.settings);
        let pos = ev.position;

        let tree = cmd.spawn_empty().id();

        let mut trunk = vec![];
        let mut canopy = vec![];

        for part in template.trunk.iter() {
            let x_i32 = pos[0] as i32 + part[0];
            let y_i32 = pos[1] as i32 + part[1];
            let z_i32 = pos[2] as i32 + part[2];

            let current = terrain.get_block_i32(x_i32, y_i32, z_i32);

            let x = x_i32 as u32;
            let y = y_i32 as u32;
            let z = z_i32 as u32;

            if current.is_oob() {
                continue;
            }

            // TODO note: structures are also considered "empty"
            if current.is_empty() || current.block == BlockType::LEAVES {
                terrain.set_block_type(x, y, z, BlockType::TREE_TRUNK);
            }

            let [chunk_idx, block_idx] = terrain.get_block_indexes(x, y, z);

            terrain.add_tree(chunk_idx, block_idx, tree);

            trunk.push([x, y, z]);
        }

        for part in template.canopy.iter() {
            let x_i32 = pos[0] as i32 + part[0];
            let y_i32 = pos[1] as i32 + part[1];
            let z_i32 = pos[2] as i32 + part[2];

            let current = terrain.get_block_i32(x_i32, y_i32, z_i32);

            let x = x_i32 as u32;
            let y = y_i32 as u32;
            let z = z_i32 as u32;

            if current.is_oob() {
                continue;
            }

            // TODO note: structures are also considered "empty"
            if current.is_empty() {
                terrain.set_block_type(x, y, z, BlockType::LEAVES);
            }

            let [chunk_idx, block_idx] = terrain.get_block_indexes(x, y, z);

            terrain.add_tree(chunk_idx, block_idx, tree);

            canopy.push([x, y, z]);
        }

        cmd.entity(tree)
            .insert((Name::new("Tree"), Tree { trunk, canopy }));
    }
}
