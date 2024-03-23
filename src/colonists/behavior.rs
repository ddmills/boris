use std::sync::Arc;

use bevy::{
    a11y::accesskit::Node,
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, EntityCommands, Query, ResMut},
    },
};

use crate::colonists::TaskIdle;

use super::{
    jobs, Fatigue, Job, JobList, Path, TaskFindBed, TaskGetJobLocation, TaskMineBlock, TaskMoveTo,
    TaskPickRandomSpot, TaskSetJob, TaskSleep,
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
    pub tree: BehaviorNodeState,
}

impl Behavior {
    pub fn new(label: &str, tree: BehaviorNode) -> Self {
        Self {
            label: String::from(label),
            tree: BehaviorNodeState::new(tree),
        }
    }
}

#[derive(Clone)]
pub enum BehaviorNode {
    /// Perform the task
    Task(Arc<dyn TaskBuilder>),
    /// Try to do the first behavior, if that fails, do the second
    Try(Box<BehaviorNode>, Box<BehaviorNode>),
    /// Return the opposite of the child
    Not(Box<BehaviorNode>),
    /// Visit all children sequentially
    Sequence(Vec<BehaviorNode>),
}

#[derive(Clone)]
pub enum BehaviorNodeState {
    None(NodeState),
    Task(NodeState, Arc<dyn TaskBuilder>),
    Try(NodeState, Box<BehaviorNodeState>, Box<BehaviorNodeState>),
    Not(NodeState, Box<BehaviorNodeState>),
    Sequence(NodeState, Vec<BehaviorNodeState>, usize),
}

#[derive(Clone, PartialEq)]
pub enum NodeState {
    Success,
    Failed,
    Executing,
    NotStarted,
}

impl BehaviorNodeState {
    pub fn new(n: BehaviorNode) -> Self {
        match n {
            BehaviorNode::Task(task) => BehaviorNodeState::Task(NodeState::NotStarted, task),
            BehaviorNode::Try(node, catch) => BehaviorNodeState::Try(
                NodeState::NotStarted,
                Box::new(BehaviorNodeState::new(*node)),
                Box::new(BehaviorNodeState::new(*catch)),
            ),
            BehaviorNode::Not(node) => BehaviorNodeState::new(*node),
            BehaviorNode::Sequence(seq) => BehaviorNodeState::Sequence(
                NodeState::NotStarted,
                seq.iter()
                    .map(|node| BehaviorNodeState::new(node.clone()))
                    .collect(),
                0,
            ),
        }
    }

    pub fn cleanup(&mut self, cmd: &mut EntityCommands) {
        if let BehaviorNodeState::Task(_, task) = self {
            task.remove(cmd)
        };
    }

    pub fn state(&self) -> &NodeState {
        match self {
            BehaviorNodeState::None(s) => s,
            BehaviorNodeState::Task(s, _) => s,
            BehaviorNodeState::Try(s, _, _) => s,
            BehaviorNodeState::Not(s, _) => s,
            BehaviorNodeState::Sequence(s, _, _) => s,
        }
    }

    fn run(&mut self, cmd: &mut EntityCommands, task_state: TaskState) -> NodeState {
        match self {
            BehaviorNodeState::None(s) => {
                *s = NodeState::Success;
                NodeState::Success
            }
            BehaviorNodeState::Task(s, task) => match *s {
                NodeState::NotStarted => {
                    task.insert(cmd);
                    *s = NodeState::Executing;
                    NodeState::Executing
                }
                NodeState::Executing => match task_state {
                    TaskState::Executing => NodeState::Executing,
                    TaskState::Success => {
                        task.remove(cmd);
                        *s = NodeState::Success;
                        NodeState::Success
                    }
                    TaskState::Failed => {
                        task.remove(cmd);
                        *s = NodeState::Failed;
                        NodeState::Failed
                    }
                },
                NodeState::Success => NodeState::Success,
                NodeState::Failed => NodeState::Failed,
            },
            BehaviorNodeState::Try(s, node, catch) => match node.state().clone() {
                NodeState::Success => {
                    *s = NodeState::Success;
                    NodeState::Success
                }
                NodeState::Failed => {
                    let b_result = catch.state();
                    match b_result {
                        NodeState::Success => {
                            *s = NodeState::Success;
                            NodeState::Success
                        }
                        NodeState::Failed => {
                            *s = NodeState::Failed;
                            NodeState::Failed
                        }
                        NodeState::Executing => {
                            *s = NodeState::Executing;
                            if NodeState::Executing != catch.run(cmd, task_state) {
                                self.run(cmd, task_state)
                            } else {
                                *s = NodeState::Executing;
                                NodeState::Executing
                            }
                        }
                        NodeState::NotStarted => {
                            *s = NodeState::Executing;
                            if NodeState::Executing != catch.run(cmd, task_state) {
                                self.run(cmd, task_state)
                            } else {
                                *s = NodeState::Executing;
                                NodeState::Executing
                            }
                        }
                    }
                }
                NodeState::Executing => {
                    *s = NodeState::Executing;
                    if NodeState::Executing != node.run(cmd, task_state) {
                        self.run(cmd, task_state)
                    } else {
                        *s = NodeState::Executing;
                        NodeState::Executing
                    }
                }
                NodeState::NotStarted => {
                    *s = NodeState::Executing;
                    if NodeState::Executing != node.run(cmd, task_state) {
                        self.run(cmd, task_state)
                    } else {
                        *s = NodeState::Executing;
                        NodeState::Executing
                    }
                }
            },
            BehaviorNodeState::Not(s, node) => match node.state().clone() {
                NodeState::Success => {
                    *s = NodeState::Failed;
                    NodeState::Failed
                }
                NodeState::Failed => {
                    *s = NodeState::Success;
                    NodeState::Success
                }
                NodeState::Executing => {
                    if NodeState::Executing != node.run(cmd, task_state) {
                        self.run(cmd, task_state)
                    } else {
                        *s = NodeState::Executing;
                        NodeState::Executing
                    }
                }
                NodeState::NotStarted => {
                    if NodeState::Executing != node.run(cmd, task_state) {
                        self.run(cmd, task_state)
                    } else {
                        *s = NodeState::Executing;
                        NodeState::Executing
                    }
                }
            },
            BehaviorNodeState::Sequence(s, seq, idx) => match s {
                NodeState::Success => NodeState::Success,
                NodeState::Failed => NodeState::Failed,
                NodeState::Executing => {
                    let Some(current) = seq.get_mut(*idx) else {
                        *s = NodeState::Failed;
                        return NodeState::Failed;
                    };

                    match current.run(cmd, task_state).clone() {
                        NodeState::NotStarted => {
                            println!("Run was called on a child node for sequence, but it did not start! {}", *idx);
                            *s = NodeState::Failed;
                            NodeState::Failed
                        }
                        NodeState::Executing => NodeState::Executing,
                        NodeState::Success => {
                            *idx += 1;
                            if *idx >= seq.len() {
                                println!("End of sequence reached successfully!");
                                *s = NodeState::Success;
                                NodeState::Success
                            } else {
                                self.run(cmd, task_state)
                            }
                        }
                        NodeState::Failed => {
                            *s = NodeState::Failed;
                            NodeState::Failed
                        }
                    }
                }
                NodeState::NotStarted => {
                    *idx = 0;

                    let Some(first) = seq.first_mut() else {
                        *s = NodeState::Failed;
                        return NodeState::Failed;
                    };

                    *s = NodeState::Executing;

                    if NodeState::Executing != first.run(cmd, task_state) {
                        self.run(cmd, task_state)
                    } else {
                        NodeState::Executing
                    }
                }
            },
        }
    }
}

#[derive(Component, Default)]
pub struct Blackboard {
    pub job: Option<Job>,
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

        let node_state = behavior
            .tree
            .run(&mut cmd.entity(has_behavior.behavior_entity), *state);

        *state = match node_state {
            NodeState::Success => TaskState::Success,
            NodeState::Failed => TaskState::Failed,
            NodeState::Executing => TaskState::Executing,
            NodeState::NotStarted => TaskState::Success,
        };

        if node_state != NodeState::Executing {
            cmd.entity(*actor).remove::<HasBehavior>();
            cmd.entity(entity).despawn();
        }

        if node_state == NodeState::Failed {
            println!("Behavior {} failed!", behavior.label);
        }
        if node_state == NodeState::Success {
            println!("Behavior {} Success!", behavior.label);
        }
        if node_state == NodeState::NotStarted {
            println!("Behavior {} Not Started?", behavior.label);
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
                BehaviorNode::Try(
                    Box::new(BehaviorNode::Sequence(vec![
                        BehaviorNode::Task(Arc::new(TaskSetJob(job))),
                        BehaviorNode::Task(Arc::new(TaskGetJobLocation)),
                        BehaviorNode::Task(Arc::new(TaskMoveTo)),
                        BehaviorNode::Task(Arc::new(TaskMineBlock(pos))),
                    ])),
                    Box::new(BehaviorNode::Task(Arc::new(TaskIdle {
                        duration_s: 5.,
                        timer: 0.,
                    }))),
                ),
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
