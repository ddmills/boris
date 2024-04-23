use std::sync::Arc;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, EntityCommands, Query, Res},
    },
    reflect::Reflect,
};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

use crate::ui::GameSpeed;

pub trait TaskBuilder: Send + Sync {
    fn insert(&self, cmd: &mut EntityCommands);
    fn remove(&self, cmd: &mut EntityCommands);
    fn label(&self) -> String;
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum TaskState {
    Executing,
    Success,
    Failed,
}

#[derive(Component, Clone)]
pub struct Actor;

#[derive(Reflect, Component, Clone, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct HasBehavior {
    pub behavior_entity: Entity,
}

#[derive(Component, Debug, Clone, Copy, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
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
    /// If the first node succeeds, do the second one, otherwise do the last one
    /// Functionally, this is the same as `Try(Sequence(A, B), C)`
    IfElse(Box<BehaviorNode>, Box<BehaviorNode>, Box<BehaviorNode>),
    /// Return the opposite of the child
    Not(Box<BehaviorNode>),
    /// Visit children sequentially, until one fails or they all succeed
    Sequence(Vec<BehaviorNode>),
    /// Visit children sequentially, until one succeeds, or they all fail
    Select(Vec<BehaviorNode>),
}

#[derive(Clone)]
pub enum BehaviorNodeState {
    Task(NodeState, Arc<dyn TaskBuilder>),
    Try(NodeState, Box<BehaviorNodeState>, Box<BehaviorNodeState>),
    IfElse(
        NodeState,
        Box<BehaviorNodeState>,
        Box<BehaviorNodeState>,
        Box<BehaviorNodeState>,
    ),
    Not(NodeState, Box<BehaviorNodeState>),
    Sequence(NodeState, Vec<BehaviorNodeState>, usize),
    Select(NodeState, Vec<BehaviorNodeState>, usize),
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
            BehaviorNode::IfElse(condition, if_node, else_node) => BehaviorNodeState::IfElse(
                NodeState::NotStarted,
                Box::new(BehaviorNodeState::new(*condition)),
                Box::new(BehaviorNodeState::new(*if_node)),
                Box::new(BehaviorNodeState::new(*else_node)),
            ),
            BehaviorNode::Select(seq) => BehaviorNodeState::Select(
                NodeState::NotStarted,
                seq.iter()
                    .map(|node| BehaviorNodeState::new(node.clone()))
                    .collect(),
                0,
            ),
        }
    }

    pub fn reset(&mut self) {
        match self {
            BehaviorNodeState::Task(s, _) => {
                *s = NodeState::NotStarted;
            }
            BehaviorNodeState::Try(s, node, catch) => {
                *s = NodeState::NotStarted;
                node.reset();
                catch.reset();
            }
            BehaviorNodeState::IfElse(s, condition, if_node, else_node) => {
                *s = NodeState::NotStarted;
                condition.reset();
                if_node.reset();
                else_node.reset();
            }
            BehaviorNodeState::Not(s, node) => {
                *s = NodeState::NotStarted;
                node.reset();
            }
            BehaviorNodeState::Sequence(s, seq, idx) => {
                *s = NodeState::NotStarted;
                seq.iter_mut().for_each(|node| node.reset());
                *idx = 0;
            }
            BehaviorNodeState::Select(s, seq, idx) => {
                *s = NodeState::NotStarted;
                seq.iter_mut().for_each(|node| node.reset());
                *idx = 0;
            }
        }
    }

    pub fn state(&self) -> &NodeState {
        match self {
            BehaviorNodeState::Task(s, _) => s,
            BehaviorNodeState::Try(s, _, _) => s,
            BehaviorNodeState::Not(s, _) => s,
            BehaviorNodeState::Sequence(s, _, _) => s,
            BehaviorNodeState::Select(s, _, _) => s,
            BehaviorNodeState::IfElse(s, _, _, _) => s,
        }
    }

    fn run(&mut self, cmd: &mut EntityCommands, task_state: TaskState) -> NodeState {
        match self {
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
                NodeState::Failed => match catch.state() {
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
                },
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
            BehaviorNodeState::IfElse(s, condition, if_node, else_node) => {
                match condition.state().clone() {
                    NodeState::Success => {
                        *s = if_node.run(cmd, task_state);
                        s.clone()
                    }
                    NodeState::Failed => {
                        *s = else_node.run(cmd, task_state);
                        s.clone()
                    }
                    NodeState::Executing => {
                        if NodeState::Executing != condition.run(cmd, task_state) {
                            self.run(cmd, task_state)
                        } else {
                            *s = NodeState::Executing;
                            NodeState::Executing
                        }
                    }
                    NodeState::NotStarted => {
                        if NodeState::Executing != condition.run(cmd, task_state) {
                            self.run(cmd, task_state)
                        } else {
                            *s = NodeState::Executing;
                            NodeState::Executing
                        }
                    }
                }
            }
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
                    *s = NodeState::Executing;
                    self.run(cmd, task_state)
                }
            },
            BehaviorNodeState::Select(s, seq, idx) => match s {
                NodeState::Success => NodeState::Success,
                NodeState::Failed => NodeState::Failed,
                NodeState::Executing => {
                    let Some(current) = seq.get_mut(*idx) else {
                        *s = NodeState::Failed;
                        return NodeState::Failed;
                    };

                    match current.run(cmd, task_state).clone() {
                        NodeState::NotStarted => {
                            println!("Run was called on a child node for select, but it did not start! {}", *idx);
                            *s = NodeState::Failed;
                            NodeState::Failed
                        }
                        NodeState::Executing => NodeState::Executing,
                        NodeState::Success => {
                            *s = NodeState::Success;
                            NodeState::Success
                        }
                        NodeState::Failed => {
                            *idx += 1;
                            if *idx >= seq.len() {
                                println!("End of sequence select failed!");
                                *s = NodeState::Failed;
                                NodeState::Failed
                            } else {
                                self.run(cmd, task_state)
                            }
                        }
                    }
                }
                NodeState::NotStarted => {
                    *idx = 0;
                    *s = NodeState::Executing;
                    self.run(cmd, task_state)
                }
            },
        }
    }
}

pub fn behavior_system(
    mut cmd: Commands,
    game_speed: Res<GameSpeed>,
    mut q_behaviors: Query<(Entity, &ActorRef, &mut Behavior, &mut TaskState)>,
    q_has_behavior: Query<&HasBehavior>,
) {
    if game_speed.is_paused {
        return;
    }

    for (entity, ActorRef(actor), mut behavior, mut state) in q_behaviors.iter_mut() {
        let Ok(has_behavior) = q_has_behavior.get(*actor) else {
            println!("Detached behavior detected? Despawning it.");
            cmd.entity(entity).despawn();
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
            cmd.entity(entity).despawn();
            cmd.entity(*actor).remove::<HasBehavior>();
        }

        // if node_state == NodeState::Failed {
        //     println!("Behavior {} failed!", behavior.label);
        //     println!("==== FAILED {}", behavior.label);
        // }
        // if node_state == NodeState::Success {
        //     println!("Behavior {} Success!", behavior.label);
        //     println!("==== SUCCESS {}", behavior.label);
        // }
        // if node_state == NodeState::NotStarted {
        //     println!("Behavior {} Not Started?", behavior.label);
        //     println!("==== NOT_STARTED {}", behavior.label);
        // }
    }
}
