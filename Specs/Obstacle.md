### **Game Design Document: Obstacle Library**

#### **1. Design Philosophy**

The core gameplay of "Moodel Herding" is managing chaos. The Moodels' unpredictable AI is the primary source of this chaos. Therefore, obstacles should not primarily *add* to the randomness. Instead, their purpose is to **structure the play space and create predictable cause-and-effect scenarios.**

Every obstacle should adhere to the following principles:

*   **Immediate Clarity:** The function of an obstacle must be visually obvious from its design. A player should know what an obstacle does the first time they see it.
*   **Predictable Interaction:** The *effect* of an obstacle on a Moodel is always consistent, even if the Moodel's path to it is not.
*   **Strategic Duality:** Most obstacles should present both a challenge and a potential tool. They can block the player's goal, but a clever player can use them to their advantage to solve the puzzle.

---

### **2. Obstacle Catalogue**

#### **Obstacle 1: The Wall**
*   **Status:** *Already Implemented.*
*   **Core Concept:** An impassable static barrier.
*   **Visual Design:** A solid, opaque rectangle with a distinct border to give it presence. Its color should be neutral and consistent across all levels.
*   **Gameplay Mechanics:** A static collider that blocks all movement.
*   **Strategic Use (Player's Perspective):**
    *   **Funneling:** Creates corridors to guide Moodels toward a specific area.
    *   **Containment:** Forms pens to trap Moodels.
    *   **Separation:** Divides the level to isolate problematic Moodels (like Rage) from more manageable ones.
*   **Design Challenge:** Overuse can make levels feel like simple mazes. Should be combined with other, more dynamic obstacles to create interesting puzzles.

---

#### **Obstacle 2: The Tar Pit**
*   **Core Concept:** A patch of ground that drastically slows Moodels.
*   **Visual Design:** A dark, semi-transparent circular or amorphous puddle with a thick border. Slow, bubbling particle effects inside make its purpose clear.
*   **Gameplay Mechanics:** A sensor area. Any Moodel entering the area has its maximum speed significantly reduced (e.g., to 25% of its normal speed). The effect is removed instantly upon exiting.
*   **Strategic Use (Player's Perspective):**
    *   **"Parking Lot":** Lure a Moodel into the pit to temporarily "store" it while dealing with others.
    *   **Safety Lane:** Create a path of tar pits next to a hazard, allowing Moodels to traverse it slowly and safely.
    *   **Rage Management:** Excellent for slowing down fast-moving Rage or Happy Moodels, making them easier to influence or contain.
*   **Design Challenge:** The slow-down amount must be balanced. Too slow, and it's just a "sticky wall." Too fast, and it's not impactful enough. It should feel like moving through molasses, not just a slight inconvenience.

---

#### **Obstacle 3: The Flow Tile**
*   **Core Concept:** A tile that pushes any Moodel on it in a fixed direction.
*   **Visual Design:** A rectangular tile with bright, clearly animated arrows scrolling across its surface, indicating the direction of flow. The tile itself could have a metallic or conveyor-belt-like texture.
*   **Gameplay Mechanics:** A sensor area that applies a constant, gentle force to any `RigidBody` on top of it. This force should be strong enough to influence a Moodel's path but not so strong that they cannot wander off the side of the tile.
*   **Strategic Use (Player's Perspective):**
    *   **Highways:** Create a long path of Flow Tiles to quickly transport Moodels across the map to a distant goal zone.
    *   **Sorters:** A "T" intersection of Flow Tiles can be used to split a group of Moodels and send them to different areas.
    *   **Denial of Entry:** Placing a Flow Tile facing away from a door effectively prevents Moodels from entering it easily.
*   **Design Challenge:** The force strength is critical. It must be a "suggestion," not a "command," allowing for some emergent behavior as Moodels try to fight the current.

---

#### **Obstacle 4: The One-Way Gate**
*   **Core Concept:** A barrier that can only be passed through from one direction.
*   **Visual Design:** A thin wall made of angled, sharp-looking holographic teeth or chevrons, all pointing in the "impassable" direction. From the "passable" direction, it looks like an open gate. When a Moodel passes through, the teeth could flash green. If one tries to go the wrong way, they flash red.
*   **Gameplay Mechanics:** Implemented as a static collider with an adjacent sensor on the "entry" side. When a Moodel enters the sensor, its collision with the gate is temporarily disabled, allowing it to pass through. From the other side, there is no sensor, so it remains a solid wall.
*   **Strategic Use (Player's Perspective):**
    *   **Puzzle Lock:** This is a fundamental puzzle tool. Getting a Moodel into a "pen" becomes a permanent decision for that level.
    *   **Safe Zones:** Create a final holding pen for a Goal Zone that Moodels can enter but not accidentally leave. This can help reduce the frustration of a "nearly won" level.
*   **Design Challenge:** The visual directionality must be unmistakable. The player should never be confused about which way is the "one way."

---

#### **Obstacle 5: The Aura Field**
*   **Core Concept:** A persistent area that slowly converts any Moodel inside to a specific mood.
*   **Visual Design:** A large, semi-transparent, colored area that shimmers or pulses gently. The color directly corresponds to the mood it imparts (yellow for Happy, blue for Sad, etc.).
*   **Gameplay Mechanics:** A sensor area. When a Moodel enters, a timer specific to that Moodel starts. If the Moodel remains in the field for a continuous duration (e.g., 2 seconds), its mood is forcibly changed to the field's mood. If it leaves, the timer resets.
*   **Strategic Use (Player's Perspective):**
    *   **Conversion Station:** A key part of a puzzle where the player must guide a Moodel into a field to get the correct mood before moving it to the goal.
    *   **Hazard:** A "Rage Field" placed in a critical path that the player must find a way to navigate around or bait other Moodels through.
*   **Design Challenge:** The conversion time is the key balancing act. Too fast, and it adds to the chaos. Too slow, and it's tedious for the player. A visual indicator (like a filling circle above the Moodel) could show the conversion progress.

---

#### **Obstacle 6: The Toggle Block**
*   **Core Concept:** A set of colored blocks that can be toggled between solid and passable by the player.
*   **Visual Design:** Two sets of blocks, e.g., "Red Blocks" and "Blue Blocks." On the map are corresponding "Red Switch" and "Blue Switch" buttons. When a block is solid, it's opaque. When passable, it's transparent and ghostly. The player's cursor highlights when over a switch.
*   **Gameplay Mechanics:** The switches are player-interactive entities. Clicking a switch sends a global event (e.g., `ToggleBlockEvent(Color::Red)`). A system listens for this event and queries for all `ToggleBlock` components of that color, enabling or disabling their `Collider` and changing their visibility.
*   **Strategic Use (Player's Perspective):**
    *   **Dynamic Mazes:** The player can actively change the level layout to open paths for desired Moodels while trapping others.
    *   **Timing Puzzles:** Requires the player to let a Moodel into an area and quickly close the path behind it before it escapes.
*   **Design Challenge:** Ensuring it's always clear to the player which switch controls which blocks. Simple color-coding is the most effective solution. The number of switches should be limited to avoid overwhelming the player.