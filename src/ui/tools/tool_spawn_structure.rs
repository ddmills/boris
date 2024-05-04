use bevy::{
    ecs::{
        entity::Entity,
        event::EventWriter,
        system::{Commands, Local, Query, Res},
    },
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
};

use crate::{
    colonists::SpawnJobBuildEvent,
    controls::Raycast,
    structures::{RemoveStructureEvent, SpawnStructureEvent, Structure, StructureMode},
    ui::{Tool, Toolbar},
};

#[derive(Default)]
pub struct StructurePlacementState {
    structure: Option<Entity>,
}

pub fn tool_spawn_structure(
    mut cmd: Commands,
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut q_structures: Query<&mut Structure>,
    mut ev_spawn_structure: EventWriter<SpawnStructureEvent>,
    mut state: Local<StructurePlacementState>,
    mut ev_remove_structure: EventWriter<RemoveStructureEvent>,
    mut ev_spawn_build_job: EventWriter<SpawnJobBuildEvent>,
) {
    let Tool::SpawnStructure(blueprint_type) = toolbar.tool else {
        if let Some(entity) = state.structure {
            ev_remove_structure.send(RemoveStructureEvent { entity });
            state.structure = None;
        }
        return;
    };

    let Some(entity) = state.structure else {
        let id = cmd.spawn_empty().id();
        state.structure = Some(id);
        ev_spawn_structure.send(SpawnStructureEvent {
            pos: raycast.adj_pos,
            entity: id,
            blueprint_type,
        });
        return;
    };

    let Ok(mut structure) = q_structures.get_mut(entity) else {
        return;
    };

    if structure.blueprint_type != blueprint_type {
        ev_remove_structure.send(RemoveStructureEvent { entity });
        state.structure = None;
        return;
    }

    if mouse_input.just_released(MouseButton::Right) {
        if structure.is_valid {
            structure.mode = StructureMode::Placed;
            structure.is_dirty = true;
            ev_spawn_build_job.send(SpawnJobBuildEvent { structure: entity });
            state.structure = None;
        } else {
            println!("invalid placement!");
        }
        return;
    }

    if key_input.just_released(KeyCode::KeyR) {
        structure.rotation += 1;
        if structure.rotation > 3 {
            structure.rotation = 0;
        };
    }

    if key_input.just_released(KeyCode::KeyF) {
        structure.is_flipped = !structure.is_flipped;
    }

    if !raycast.is_adj_hit {
        structure.is_valid = false;
        return;
    }

    structure.position = raycast.adj_pos;
    structure.is_dirty = true;
}
