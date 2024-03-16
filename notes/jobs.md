**Job**

Something that the player wants done. A job can only be claimed by one colonist at a time.

Properties
    - urgency (player assigned number)
    - created
    - 

Examples:
    - cook a meal
    - kill a monster
    - mine some rock
    - chop a tree
    - pick some berries


**Task**

A smaller piece of a job. Moast tasks should have a _location_. This location can be used to determine
distance and priority. Moving somehwere is not a task in itself.

Properties
    - location

Examples:
    - pick up object
    - store object
    - use workstation
    - interact with object
    - wait
    - equip object


**Prerequisites**

A requirement in order for a task to be started. Prerequisites are checked in reverse order. "Moving
to the task" is implied.

Examples:

- (job) Cook a meal
    - (task) pick up meat
        - (prerequisite) able to carry items
    - (task) use stove workstation
        - (prerequisite) have cookable item in inventory

- (job) store brick
    - (task) pick up brick
        - (prerequisite) able to carry items
    - (task) put brick in storage
        - (prerequisite) have brick in inventory


**Thoughts**

Some tasks might require a _specific_ instance of an object, while others could be more general. For
example, if you have an object labelled for storage, the task "put object in storage" would be
referring to the one defined by the job. A more general job might be "cook meal", which could use
any cookable item.

1. Keep all jobs in a list
    - must be able to determine the priority of a job for a given worker
    - must be able to sort jobs by priority for a given worker
    - priority is determined by worker skill, distance, and if they are close to completing the job already. (i.e, if you a holding a pickaxe, you are closer to completing a digging job)
2. Different tasks need very different data sets
    - A pathing task requires knowledge of the terrain
    - A stockpiling task needs knowledge of stockpiles
    - A "Pick up item" task needs knowledge of inventories
    - A "kill enemy" task needs knowledge of where an enemy is
3. Tasks have precondtions that need different data sets. Preconditions have different results depending on the worker!
    - In order to chop a tree, you must be holding an axe and standing next to it
    - In order to "pick up axe", you must be standing next to it
    - In order to be standing next to an axe, you must be able to path to that location


- (job) chop tree
    - (task) aquire axe
        - (prerequisite) able path to location of _any_ axe
    - (task) equip axe 
        - (prerequisite) have _any_ axe in inventory
    - (task) swing axe
        - (prerequisite) have _any_ axe equipped
        - (prerequisite) able path to location of _the_ tree

Work this list in reverse, i.e,
    1. "Do i have an axe equipped, and can i reach the tree?" Yes? Swing Axe, Else? Equip Axe or Cannot do job
    2. "Do i have an axe in my inventory?" Yes? Equip the axe, Else? Aquire Axe
    3. "Can i path to the location of an axe?" Yes? Aquire axe, Else? Cannot do job

The issue i'm running into is that all of these different tasks and prereqs require different knowledge/data. They need to know about worker inventories, stockpiles, map distances, accessibility, etc. I can answer these questions, but they require pulling in a lot of different ECS queries and resources. My brain keeps going to OOP. Then I was thinking, perhaps i have a system for every type of task, this works great for executing them, but does not answer the priority question. To determine priority, i have to look at all tasks and all preconditions, which doesn't work if that logic is in a different system. Note i've also excluded "path to object" as a task, I was thinking that might be implied for most tasks


- (job) mine block
    - (task) aquire pickaxe
        - (prerequisite) able path to location of _any_ pickaxe
    - (task) equip pickaxe 
        - (prerequisite) have _any_ pickaxe in inventory
    - (task) swing pickaxe
        - (prerequisite) have _any_ pickaxe equipped
        - (prerequisite) able path to location of _the_ block

simple

Job - MineBlock(xyz)
    Task - Go next to xyz
        Prereq - next to XYZ is accessible
    Task - SwingPickaxe
        Prereq - standing next to XYZ
