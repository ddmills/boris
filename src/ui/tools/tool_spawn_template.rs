use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        event::EventWriter,
        system::{Commands, Local, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    input::{mouse::MouseButton, ButtonInput},
    math::{Quat, Vec3},
    render::view::Visibility,
    transform::{self, components::Transform},
};

use crate::{
    controls::Raycast,
    furniture::{SpawnTemplateEvent, TemplateInstance, TemplateTileRequirement, Templates},
    ui::{Tool, Toolbar},
    Terrain,
};

#[derive(Default)]
pub struct TemplatePlacementState {
    entity: Option<Entity>,
    rotation: u8,
    flipped: bool,
}

fn apply_transforms(center: [i32; 3], point: [i32; 3], r: u8, f: bool) -> [i32; 3] {
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    let z = point[2] - center[2];

    let [rx, ry, rz] = match r {
        1 => [z, y, -x],
        2 => [-x, y, -z],
        3 => [-z, y, x],
        _ => [x, y, z],
    };

    match f {
        true => [rx, ry, -rz],
        false => [rx, ry, rz],
    }
}

pub fn tool_spawn_template(
    mut cmd: Commands,
    toolbar: Res<Toolbar>,
    terrain: Res<Terrain>,
    raycast: Res<Raycast>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut q_transforms: Query<&mut Transform>,
    mut q_templates: Query<&mut TemplateInstance>,
    mut ev_spawn_template: EventWriter<SpawnTemplateEvent>,
    mut state: Local<TemplatePlacementState>,
    templates: Res<Templates>,
) {
    let Tool::SpawnTemplate(template_type) = toolbar.tool else {
        if let Some(entity) = state.entity {
            cmd.entity(entity).despawn_recursive();
            state.entity = None;
            state.flipped = false;
            state.rotation = 0;

            for interface in templates.interfaces.iter() {
                cmd.entity(*interface).insert(Visibility::Hidden);
            }
        }
        return;
    };

    let Some(entity) = state.entity else {
        let id = cmd.spawn_empty().id();
        println!("spawn entity {}", id.index());
        state.entity = Some(id);
        ev_spawn_template.send(SpawnTemplateEvent {
            pos: raycast.adj_pos,
            entity: id,
            template_type,
        });
        return;
    };

    let Ok(mut template) = q_templates.get_mut(entity) else {
        return;
    };

    let Ok(mut transform) = q_transforms.get_mut(entity) else {
        return;
    };

    if mouse_input.just_released(MouseButton::Right) {
        state.rotation += 1;
        if state.rotation > 3 {
            state.rotation = 0;
        };
        println!("build it now {}, {}", state.rotation, state.flipped);
    }

    if mouse_input.just_released(MouseButton::Left) {
        state.flipped = !state.flipped;
        println!("build it now {}, {}", state.rotation, state.flipped);
    }

    if !raycast.is_adj_hit {
        template.is_valid = false;
        return;
    }

    let Some(data) = templates.templates.get(&template.template_type) else {
        println!("Missing template type");
        return;
    };

    template.position = raycast.adj_pos;

    transform.translation.x = template.position[0] as f32 + 0.5;
    transform.translation.y = template.position[1] as f32;
    transform.translation.z = template.position[2] as f32 + 0.5;

    if state.flipped {
        transform.scale = Vec3::new(1., 1., 1.);
        transform.rotation = match state.rotation {
            1 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
            2 => Quat::from_rotation_y(std::f32::consts::PI),
            3 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            _ => Quat::from_rotation_y(0.),
        };
    } else {
        transform.scale = Vec3::new(1., 1., -1.);
        transform.rotation = match state.rotation {
            1 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            2 => Quat::from_rotation_y(std::f32::consts::PI),
            3 => Quat::from_rotation_y(std::f32::consts::PI + std::f32::consts::FRAC_PI_2),
            _ => Quat::from_rotation_y(0.),
        };
    }

    let center = [
        data.center[0] as i32,
        data.center[1] as i32,
        data.center[2] as i32,
    ];

    template.is_valid = data.tiles.iter().enumerate().all(|(idx, tile)| {
        let tr = apply_transforms(center, tile.position, state.rotation, state.flipped);
        let pos_i32 = [
            template.position[0] as i32 + tr[0],
            template.position[1] as i32 + tr[1],
            template.position[2] as i32 + tr[2],
        ];

        // if let Some(interface) = tile.interface {
        //     let interface_e = templates.interfaces.get(idx).unwrap();
        //     let mut interface_t = q_transforms.get_mut(*interface_e).unwrap();

        //     cmd.entity(*interface_e).insert(Visibility::Visible);

        //     interface_t.translation = Vec3::new(
        //         pos_i32[0] as f32 + 0.5,
        //         pos_i32[1] as f32,
        //         pos_i32[2] as f32 + 0.5,
        //     );
        //     interface_t.rotation = interface.direction.to_quat(state.flipped);
        // }

        let block = terrain.get_block_i32(pos_i32[0], pos_i32[1], pos_i32[2]);

        if block.is_oob() {
            return false;
        }

        tile.requirements.iter().all(|flag| match flag {
            TemplateTileRequirement::EMPTY => block.is_empty(),
            TemplateTileRequirement::ATTACHABLE => block.is_attachable(),
            TemplateTileRequirement::WALKABLE => {
                let below = terrain.get_block_i32(pos_i32[0], pos_i32[1] - 1, pos_i32[2]);

                below.is_walkable()
            }
            _ => true,
        })
    });

    // if template.is_valid && mouse_input.just_released(MouseButton::Left) {
    //     println!("build it now {}, {}", state.rotation, state.flipped);
    // }
}
