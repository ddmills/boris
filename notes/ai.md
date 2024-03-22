behavior tree


### Actors
Actors are entities that will be moving around in your world. Tag them with the `Actor` component.

### Tasks
Tasks are components that implement `TaskBuilder`. These are composed to build out behaviors. Example:
```rs
#[derive(Component, Clone, TaskBuilder)]
pub struct TaskIdle {
    pub timer: f32,
    pub duration_s: f32,
}
```

### Behaviors
Behaviors Entities piggy back on Actor. They are composed of the following components:
    - `ActorRef` - A component that references the actor
    - `TaskState` - An enum that describe current task (Success, Failure, Executing)
    - `Behavior` - A behavior tree of nodes and tasks
    - `Blackboard` - A place to share data between tasks for this behavior
    - A task component - The currently executing task will be inserted on this entity

### Task Systems
For each task, you can simply define a bevy system. This grants you access to any bevy resource or entities you need.

A task system might look like this
```rs
pub fn task_sleep(
    time: Res<Time>,
    mut q_fatigues: Query<&mut Fatigue>,
    mut q_behavior: Query<(&ActorRef, &Blackboard, &mut TaskState), With<TaskSleep>>,
) {
    for (ActorRef(entity), blackboard, mut state) in q_behavior.iter_mut() {
        let Ok(mut fatigue) = q_fatigues.get_mut(*entity) else {
            println!("Actor entity does not have a fatigue");
            *state = TaskState::Failed;
            continue;
        };

        if fatigue.value > 0. {
            fatigue.value -= time.delta_seconds() * 40.;
        }

        if fatigue.value <= 0. {
            println!("slept in bed {}", blackboard.bed); // get access to your blackboard data
            fatigue.value = 0.;
            *state = TaskState::Success;
        }
    }
}
```

### Behavior Systems

Behaviors are created and then assigned to Actors. You can find all actors without a behavior:

```rs
Query<Entity, (With<Actor>, Without<HasBehavior>)>,
```

You can attach a behavior to an actor, like so:

```rs
let b_entity = commands
    .spawn((
        Blackboard::default(),
        TaskState::Success,
        ActorRef(actor),
        Behavior::new(
            "Sleep",
            BehaviorNode::Sequence(vec![
                BehaviorNode::Task(Arc::new(TaskFindBed)),
                BehaviorNode::Task(Arc::new(TaskSleep)),
            ]),
        )
    ))
    .id();

commands.entity(actor).insert(HasBehavior {
    behavior_entity: b_entity,
});
```
