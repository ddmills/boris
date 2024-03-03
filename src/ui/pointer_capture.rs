use bevy::{
    ecs::{
        component::Component,
        query::{With, Without},
        system::{Query, ResMut, Resource},
    },
    ui::{Interaction, Node},
};

#[derive(Resource, Default)]
pub struct Ui {
    pub pointer_captured: bool,
}

#[derive(Component)]
pub struct NoPointerCapture;

#[allow(clippy::type_complexity)]
pub fn ui_capture_pointer(
    mut ui_handling: ResMut<Ui>,
    interaction_query: Query<&Interaction, (With<Node>, Without<NoPointerCapture>)>,
) {
    ui_handling.pointer_captured = interaction_query
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
}
