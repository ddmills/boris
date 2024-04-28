use bevy::{
    ecs::{
        entity::Entity,
        event::EventWriter,
        system::{Commands, Local, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
};

use crate::{
    controls::Raycast,
    furniture::{Blueprint, SpawnBlueprintEvent},
    ui::{Tool, Toolbar},
};

#[derive(Default)]
pub struct BlueprintPlacementState {
    blueprint: Option<Entity>,
}

pub fn tool_spawn_template(
    mut cmd: Commands,
    toolbar: Res<Toolbar>,
    raycast: Res<Raycast>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut q_blueprints: Query<&mut Blueprint>,
    mut ev_spawn_blueprint: EventWriter<SpawnBlueprintEvent>,
    mut state: Local<BlueprintPlacementState>,
) {
    let Tool::SpawnBlueprint(template_type) = toolbar.tool else {
        if let Some(entity) = state.blueprint {
            cmd.entity(entity).despawn_recursive();
            state.blueprint = None;
        }
        return;
    };

    let Some(entity) = state.blueprint else {
        let id = cmd.spawn_empty().id();
        state.blueprint = Some(id);
        ev_spawn_blueprint.send(SpawnBlueprintEvent {
            pos: raycast.adj_pos,
            entity: id,
            template_type,
        });
        return;
    };

    let Ok(mut blueprint) = q_blueprints.get_mut(entity) else {
        return;
    };

    if blueprint.template_type != template_type {
        cmd.entity(entity).despawn_recursive();
        state.blueprint = None;
        return;
    }

    if mouse_input.just_released(MouseButton::Right) {
        if blueprint.is_valid {
            blueprint.is_placed = true;
            blueprint.is_dirty = true;
            state.blueprint = None;
        } else {
            println!("invalid placement!");
        }
        return;
    }

    if key_input.just_released(KeyCode::KeyR) {
        blueprint.rotation += 1;
        if blueprint.rotation > 3 {
            blueprint.rotation = 0;
        };
    }

    if key_input.just_released(KeyCode::KeyF) {
        blueprint.is_flipped = !blueprint.is_flipped;
    }

    if !raycast.is_adj_hit {
        blueprint.is_valid = false;
        return;
    }

    blueprint.position = raycast.adj_pos;
}
