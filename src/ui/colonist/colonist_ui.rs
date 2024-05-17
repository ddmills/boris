use bevy::ecs::{
    entity::Entity,
    event::{Event, EventReader},
};
use bevy_mod_picking::{
    events::{Down, Pointer},
    prelude::ListenerInput,
};

#[derive(Event)]
pub struct ColonistClickedEvent(Entity, f32);

impl From<ListenerInput<Pointer<Down>>> for ColonistClickedEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        ColonistClickedEvent(event.target, event.hit.depth)
    }
}

pub fn on_colonist_clicked(mut ev_colonist_clicked: EventReader<ColonistClickedEvent>) {
    for ev in ev_colonist_clicked.read() {
        println!("Colonist clicked!");
    }
}
