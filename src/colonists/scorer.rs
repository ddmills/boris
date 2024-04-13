use std::sync::Arc;

use bevy::{
    app::{Plugin, PreUpdate},
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        system::{Commands, EntityCommands, Query},
    },
    prelude::App,
};

use crate::colonists::{ScorerBuild, ScorerMine, ScorerWander};

use super::{ActorRef, Behavior};

#[derive(Clone, Component, Debug)]
pub struct Score(pub f32);

#[derive(Component, Clone)]
pub struct Thinker {
    pub score_builders: Vec<Arc<dyn ScorerBuilder>>,
}

#[derive(Component)]
pub struct Scorers {
    pub scorers: Vec<Entity>,
}

#[derive(Component)]
pub struct ScoreBuilderRef(pub usize);

#[bevy_trait_query::queryable]
pub trait ScorerBuilder: Send + Sync {
    fn insert(&self, cmd: &mut EntityCommands);
    fn label(&self) -> String;
    fn build(&self) -> Behavior;
}

pub struct ScorerPlugin;

impl Plugin for ScorerPlugin {
    fn build(&self, app: &mut App) {
        use bevy_trait_query::RegisterExt;

        app.register_component_as::<dyn ScorerBuilder, ScorerMine>()
            .register_component_as::<dyn ScorerBuilder, ScorerBuild>()
            .register_component_as::<dyn ScorerBuilder, ScorerWander>()
            .add_systems(PreUpdate, spawn_scorers);
    }
}

pub fn spawn_scorers(
    mut cmd: Commands,
    mut q_thinkers: Query<(Entity, &mut Thinker), Without<Scorers>>,
) {
    for (actor, thinker) in q_thinkers.iter_mut() {
        let scorers = thinker
            .score_builders
            .iter()
            .enumerate()
            .map(|(idx, builder)| {
                let scorer = cmd
                    .spawn((ActorRef(actor), ScoreBuilderRef(idx), Score(0.)))
                    .id();
                let mut e_cmd = cmd.entity(scorer);

                builder.insert(&mut e_cmd);
                scorer
            })
            .collect();

        cmd.entity(actor).insert(Scorers { scorers });
    }
}
