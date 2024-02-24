use bevy::{
    ecs::{
        event::{Event, EventReader, EventWriter},
        system::{Commands, Res, ResMut, Resource},
    },
    input::mouse::MouseWheel,
};

use crate::block::world::terrain::Terrain;

#[derive(Resource)]
pub struct TerrainSlice {
    y: u32,
    min: u32,
    max: u32,
    is_enabled: bool,
}

impl TerrainSlice {
    pub fn set_value(&mut self, v: i32) -> u32 {
        self.y = v.clamp(self.min as i32, self.max as i32) as u32;
        return self.get_value();
    }

    pub fn get_value(&self) -> u32 {
        return if self.is_enabled { self.y } else { self.max };
    }
}

pub fn setup_terrain_slice(mut commands: Commands, terrain: Res<Terrain>) {
    let max = terrain.chunk_size * terrain.chunk_count_y;
    commands.insert_resource(TerrainSlice {
        y: 16,
        max: max,
        min: 0,
        is_enabled: true,
    });
}

#[derive(Event)]
pub struct TerrainSliceChanged {
    prev: u32,
    value: u32,
}

pub fn scroll_events(
    mut scroll_evt: EventReader<MouseWheel>,
    mut terrain_slice: ResMut<TerrainSlice>,
    mut ev_terrain_slice: EventWriter<TerrainSliceChanged>,
) {
    for ev in scroll_evt.read() {
        match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                let cur_slice = terrain_slice.get_value();
                let scroll = ev.y as i32;
                let slice = terrain_slice.y as i32;
                terrain_slice.set_value(slice + scroll);
                ev_terrain_slice.send(TerrainSliceChanged {
                    prev: cur_slice,
                    value: terrain_slice.get_value(),
                });
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {}
        }
    }
}
