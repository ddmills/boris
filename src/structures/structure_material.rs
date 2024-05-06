use bevy::{asset::Handle, ecs::system::Commands};

use bevy::{
    asset::Assets,
    ecs::system::{Query, ResMut},
    render::{color::Color, view::Visibility},
};

use crate::rendering::BasicMaterial;

use super::{Structure, StructureGuide, StructureMode};

pub const BLUEPRINT_COLOR_VALID: Color = Color::rgb(0.192, 0.51, 0.90);
pub const BLUEPRINT_COLOR_HOTSPOTS_INVALID: Color = Color::rgb(0.9, 0.9, 0.1);
pub const BLUEPRINT_COLOR_INVALID: Color = Color::rgb(0.9, 0.1, 0.1);

pub fn structure_material_update(
    mut cmd: Commands,
    q_structures: Query<(&Structure, &Handle<BasicMaterial>)>,
    q_guides: Query<(&StructureGuide, &Handle<BasicMaterial>)>,
    mut basic_materials: ResMut<Assets<BasicMaterial>>,
) {
    for (structure, material_handle) in q_structures.iter() {
        for guide_e in structure.guides.iter() {
            let Ok((guide, guide_mat_handle)) = q_guides.get(*guide_e) else {
                continue;
            };

            let Some(guide_material) = basic_materials.get_mut(guide_mat_handle) else {
                continue;
            };

            if matches!(structure.mode, StructureMode::Placed | StructureMode::Built) {
                cmd.entity(*guide_e).insert(Visibility::Hidden);
                continue;
            }

            if guide.is_hotspot {
                if structure.is_valid && guide.is_valid {
                    cmd.entity(*guide_e).insert(Visibility::Inherited);
                } else {
                    cmd.entity(*guide_e).insert(Visibility::Hidden);
                }
            }

            guide_material.color = match structure.is_valid {
                true => match structure.is_hotspots_valid {
                    true => BLUEPRINT_COLOR_VALID,
                    false => BLUEPRINT_COLOR_HOTSPOTS_INVALID,
                },
                false => BLUEPRINT_COLOR_INVALID,
            };
        }

        let Some(material) = basic_materials.get_mut(material_handle) else {
            continue;
        };

        if structure.is_built() {
            material.color = Color::WHITE;
            material.enable_slots = true;
            material.is_lit = true;
            continue;
        }

        material.color = match structure.is_valid {
            true => match structure.is_hotspots_valid {
                // true => Color::rgb_from_array([0.435, 0.656, 0.851]),
                true => BLUEPRINT_COLOR_VALID,
                false => BLUEPRINT_COLOR_HOTSPOTS_INVALID,
            },
            false => BLUEPRINT_COLOR_INVALID,
        };
    }
}
