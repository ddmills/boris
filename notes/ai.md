behavior tree

Actors will have a Brain component

Brains will define a generic Blackboard
    - will hold all types of data, including path
    - will hold current action result

Brains will create Behaviors with Action components
    - these will be built using ActionBuilders
    - They will have an Actor(Entity) component that references the brain entity

an Act must be defined as a component on the entity, ex:
    - FindBedAct
    - MoveToAct
    - SleepAct


Sequence(ActFindBed, MoveToAct, ActSleep)

Actors start without any behavior
    - A dedicated system will assign a behavior, to entities without one, for now
        - this could be done using scores
    - 