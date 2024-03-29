# Partitions

These definitions are in order from small to large.

**Navigation Flags**:
A unique set of flags for navigation purposes. Represented using a bitset.

**Partition**:
A continous collection of blocks that all have the same Navigation Flags.
A partition cannot be larger than a chunk (16x16x16). A partition also keeps
track of neighboring partitions. Partitions keep track of their extents, to use as a navigation heuristic.

**Region**:
A continous set of partitions with matching Navigation Flags.
Regions track their neighboring regions as well.

**Navigation Group**:
A navigation group is a set of connected Regions that match the navigation flags defined for the group. Navigation groups can overlap.

**Navigation Graph**
The navigation graph holds all of the partitions, regions, and navigation groups. It also provides methods for getting access to them.
