use bevy::{
    core::Name,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    hierarchy::{BuildChildren, Children},
    math::{Quat, Vec3},
    render::view::Visibility,
    transform::components::Transform,
};
use task_derive::TaskBuilder;

use crate::colonists::{ActorRef, Blackboard, Item, TaskBuilder, TaskState};

#[derive(Component, Clone, TaskBuilder)]
pub struct TaskItemEquip;

pub fn task_item_equip(
    mut cmd: Commands,
    q_names: Query<&Name>,
    q_children: Query<&Children>,
    mut q_items: Query<&mut Transform, With<Item>>,
    mut q_behavior: Query<(&ActorRef, &mut TaskState, &Blackboard), With<TaskItemEquip>>,
) {
    for (ActorRef(actor), mut state, blackboard) in q_behavior.iter_mut() {
        let Some(item_entity) = blackboard.item else {
            println!("Cannot equip item, no item on blackboard");
            *state = TaskState::Failed;
            continue;
        };

        let Ok(mut item_transform) = q_items.get_mut(item_entity) else {
            println!("Cannot equip item, no transform?");
            *state = TaskState::Failed;
            continue;
        };

        let Some(grasper) = get_child_by_name_recursive(actor, "Grasp.R", &q_names, &q_children)
        else {
            println!("Cannot equip item, no grasper child");
            *state = TaskState::Success;
            continue;
        };

        item_transform.translation = Vec3::ZERO;
        item_transform.rotation = Quat::IDENTITY;
        item_transform.scale = Vec3::ONE;
        let mut item_cmds = cmd.entity(item_entity);
        item_cmds.insert(Visibility::Visible);
        item_cmds.set_parent(grasper);

        *state = TaskState::Success;
    }
}

pub fn get_child_by_name_recursive(
    entity: &Entity,
    name: &str,
    q_names: &Query<&Name>,
    q_children: &Query<&Children>,
) -> Option<Entity> {
    let Ok(children) = q_children.get(*entity) else {
        return None;
    };

    for child in children.iter() {
        if let Ok(child_name) = q_names.get(*child) {
            if child_name.as_str().eq(name) {
                return Some(*child);
            }
        };

        let in_child = get_child_by_name_recursive(child, name, q_names, q_children);

        if in_child.is_some() {
            return in_child;
        }
    }

    None
}
