use std::sync::Arc;

use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, EntityCommands, Query, ResMut},
};

use crate::colonists::TaskIdle;

use super::{
    jobs, Fatigue, JobList, Path, TaskFindBed, TaskGetJobLocation, TaskMineBlock, TaskMoveTo,
    TaskPickRandomSpot, TaskSleep,
};

pub trait TaskBuilder: Send + Sync {
    fn insert(&self, cmd: &mut EntityCommands);
    fn remove(&self, cmd: &mut EntityCommands);
    fn label(&self) -> String;
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum TaskState {
    Executing,
    Success,
    Failed,
}

#[derive(Component, Clone)]
pub struct Actor;

#[derive(Component, Clone)]
pub struct HasBehavior {
    pub behavior_entity: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ActorRef(pub Entity);

#[derive(Component, Clone)]
pub struct Behavior {
    pub label: String,
    pub state: BehaviorNodeState,
}

impl Behavior {
    pub fn new(label: &str, node: BehaviorNode) -> Self {
        Self {
            label: String::from(label),
            state: BehaviorNodeState::new(node),
        }
    }
}

#[derive(Clone)]
pub enum BehaviorNode {
    /// Perform the task
    Task(Arc<dyn TaskBuilder>),
    /// Return the opposite of the child
    Not(Box<BehaviorNode>),
    /// Visit all children sequentially
    Sequence(Vec<BehaviorNode>),
}

#[derive(Clone)]
pub enum BehaviorNodeState {
    None,
    Task(Arc<dyn TaskBuilder>),
    Not(Box<BehaviorNode>),
    Sequence(Vec<BehaviorNode>, usize, Box<BehaviorNodeState>),
}

impl BehaviorNodeState {
    pub fn new(node: BehaviorNode) -> Self {
        match node {
            BehaviorNode::Task(n) => BehaviorNodeState::Task(n),
            BehaviorNode::Not(n) => BehaviorNodeState::Not(n),
            BehaviorNode::Sequence(seq) => {
                BehaviorNodeState::Sequence(seq, 0, Box::new(BehaviorNodeState::None))
            }
        }
    }

    pub fn cleanup(&mut self, cmd: &mut EntityCommands) {
        if let BehaviorNodeState::Task(task) = self {
            task.remove(cmd)
        };
    }

    pub fn run(&mut self, cmd: &mut EntityCommands, task_state: &mut TaskState) {
        match self {
            BehaviorNodeState::None => {}
            BehaviorNodeState::Task(t) => {
                println!("insert->{}", t.label());
                t.insert(cmd);
                *task_state = TaskState::Executing;
            }
            BehaviorNodeState::Not(_) => {
                *task_state = match task_state {
                    TaskState::Executing => TaskState::Executing,
                    TaskState::Success => TaskState::Failed,
                    TaskState::Failed => TaskState::Success,
                };
            }
            BehaviorNodeState::Sequence(seq, idx, cursor) => {
                if *idx >= seq.len() {
                    return;
                }

                (*cursor).cleanup(cmd);

                if *task_state != TaskState::Success {
                    return;
                }

                let next_task = seq.get(*idx).unwrap();
                let mut next_state = BehaviorNodeState::new(next_task.clone());
                next_state.run(cmd, task_state);
                **cursor = next_state;

                *idx += 1;
            }
        }
    }
}

#[derive(Component, Default)]
pub struct Blackboard {
    pub bed: u8,
    pub move_goals: Vec<[u32; 3]>,
    pub path: Option<Path>,
}

pub fn behavior_system(
    mut cmd: Commands,
    mut q_behaviors: Query<(Entity, &ActorRef, &mut Behavior, &mut TaskState)>,
    q_has_behavior: Query<&HasBehavior>,
) {
    for (entity, ActorRef(actor), mut behavior, mut state) in q_behaviors.iter_mut() {
        let Ok(has_behavior) = q_has_behavior.get(*actor) else {
            println!("Detached behavior detected?");
            continue;
        };

        if *state == TaskState::Executing {
            continue;
        }

        behavior
            .state
            .run(&mut cmd.entity(has_behavior.behavior_entity), &mut state);

        if *state != TaskState::Executing {
            cmd.entity(*actor).remove::<HasBehavior>();
            cmd.entity(entity).despawn();
        }

        if *state == TaskState::Failed {
            println!("Behavior {} failed!", behavior.label);
        }
        if *state == TaskState::Success {
            println!("Behavior {} Success!", behavior.label);
        }
    }
}

pub fn behavior_pick_system(
    mut commands: Commands,
    q_actors: Query<(Entity, &Fatigue), (With<Actor>, Without<HasBehavior>)>,
    mut jobs: ResMut<JobList>,
) {
    for (actor, fatigue) in q_actors.iter() {
        let behavior = get_behavior(fatigue, &mut jobs);

        let b_entity = commands
            .spawn((
                Blackboard::default(),
                TaskState::Success,
                ActorRef(actor),
                behavior,
            ))
            .id();

        commands.entity(actor).insert(HasBehavior {
            behavior_entity: b_entity,
        });
    }
}

pub fn get_behavior(fatigue: &Fatigue, jobs: &mut JobList) -> Behavior {
    if fatigue.value > 75. {
        return Behavior::new(
            "Sleep",
            BehaviorNode::Sequence(vec![
                BehaviorNode::Task(Arc::new(TaskFindBed)),
                BehaviorNode::Task(Arc::new(TaskSleep)),
            ]),
        );
    }

    if let Some(job) = jobs.pop() {
        return match job {
            jobs::Job::Mine(pos) => Behavior::new(
                "Mine",
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Task(Arc::new(TaskGetJobLocation(job))),
                    BehaviorNode::Task(Arc::new(TaskMoveTo)),
                    BehaviorNode::Task(Arc::new(TaskMineBlock(pos))),
                ]),
            ),
        };
    }

    Behavior::new(
        "Wander",
        BehaviorNode::Sequence(vec![
            BehaviorNode::Task(Arc::new(TaskPickRandomSpot)),
            BehaviorNode::Task(Arc::new(TaskMoveTo)),
            BehaviorNode::Task(Arc::new(TaskIdle {
                duration_s: 2.,
                timer: 0.,
            })),
        ]),
    )
}
