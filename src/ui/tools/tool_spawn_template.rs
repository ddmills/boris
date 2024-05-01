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
    furniture::{Blueprint, BlueprintMode, RemoveBlueprintEvent, SpawnBlueprintEvent},
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
    mut ev_remove_blueprint: EventWriter<RemoveBlueprintEvent>,
    mut ev_spawn_build_job: EventWriter<SpawnJobBuildEvent>,
) {
    let Tool::SpawnBlueprint(template_type) = toolbar.tool else {
        if let Some(entity) = state.blueprint {
            ev_remove_blueprint.send(RemoveBlueprintEvent { entity });
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
        ev_remove_blueprint.send(RemoveBlueprintEvent { entity });
        state.blueprint = None;
        return;
    }

    if mouse_input.just_released(MouseButton::Right) {
        if blueprint.is_valid {
            blueprint.mode = BlueprintMode::Placed;
            blueprint.is_dirty = true;
            ev_spawn_build_job.send(SpawnJobBuildEvent { blueprint: entity });
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
    blueprint.is_dirty = true;
}
