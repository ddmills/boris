use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    time::Time,
};

#[derive(Component, Default)]
pub struct Fatigue {
    pub value: f32,
    pub per_second: f32,
}

pub fn fatigue_system(time: Res<Time>, mut q_fatigues: Query<&mut Fatigue>) {
    for mut fatigue in q_fatigues.iter_mut() {
        fatigue.value += fatigue.per_second * time.delta_seconds();

        if fatigue.value >= 100. {
            fatigue.value = 100.;
        }
    }
}
