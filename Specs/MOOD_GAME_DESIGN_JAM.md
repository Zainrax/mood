# Mood - Simplified Game Design (Game Jam Version)

## Core Concept
Simple top-down game where players use shape tools to influence Moodel movement and achieve target mood ratios.

## Moodel Types & Movement

### 1. Calm Moodel (Blue) - Uses MoodelCalm.png
- **Movement**: Slow, straight lines with occasional gentle turns
- **Speed**: 100 units/second
- **Behavior**: Attracted to quiet areas, repelled by chaos

### 2. Happy Moodel (Yellow) - Uses MoodelHappy.png
- **Movement**: Bouncy, energetic with small hops
- **Speed**: 200 units/second
- **Behavior**: Attracted to groups, bounces off walls cheerfully

### 3. Rage Moodel (Red) - Uses MoodelRage.png
- **Movement**: Fast, aggressive straight lines
- **Speed**: 300 units/second
- **Behavior**: Charges forward, bounces hard off obstacles

### 4. Sad Moodel (Purple) - Uses MoodelSad.png
- **Movement**: Very slow, drifts aimlessly
- **Speed**: 75 units/second
- **Behavior**: Sinks toward bottom, avoids crowds

## Simple Conversion Rules
Moodels change based on their surroundings:
- **Isolated** → Sad (loneliness)
- **In small groups (2-3)** → Calm (comfortable)
- **In medium groups (4-6)** → Happy (social energy)
- **In large crowds (7+)** → Rage (overwhelmed)

## Player Shape Tools

### 1. Line Tool (Q Key)
- **Shape**: Straight line barrier
- **Effect**: Moodels bounce off at 90° angles
- **Use**: Redirect movement, separate groups
- **Duration**: 5 seconds

### 2. Box Tool (W Key)
- **Shape**: Rectangular containment area
- **Effect**: Traps Moodels inside, creates forced grouping
- **Use**: Force specific group sizes for conversions
- **Duration**: 8 seconds

### 3. Circle Tool (E Key)
- **Shape**: Circular attraction/repulsion field
- **Effect**: Click to attract, Hold to repel
- **Use**: Gather or scatter Moodels
- **Duration**: 3 seconds per use

## Victory Conditions

## Gameplay Flow
1. **Level Start**: Moodels spawn randomly across screen
2. **Player Goal**: Use shape tools to manipulate movement and grouping
3. **Conversion**: Moodels change mood based on proximity to others
4. **Victory**: Achieve target ratio within time limit

### Example Level Goals
- Level 1: 70% Happy (tutorial - create medium groups)
- Level 2: 50% Calm, 30% Happy, 20% Sad
- Level 3: Equal 25% distribution
- Level 4: 60% Calm, 40% Sad (separate everyone)
- Level 5: 80% Rage (create huge crowds)

### Timer
- 45 seconds per level
- Real-time ratio display with colored bars
- Win when within 5% of target ratio

## Implementation Priority (4 Days)

### Day 1: Core Systems
- [ ] 4 Moodel types with sprites and movement patterns
- [ ] Proximity-based mood conversion system
- [ ] Basic UI showing current mood ratios

### Day 2: Shape Tools
- [ ] Line tool - click & drag to create barriers
- [ ] Box tool - click & drag to create containment areas
- [ ] Circle tool - click for attraction field

### Day 3: Game Loop
- [ ] Level system with target ratios
- [ ] Timer and victory conditions
- [ ] 5 tutorial/progression levels

### Day 4: Polish & Balance
- [ ] Visual feedback for tools and conversions
- [ ] Sound effects for tool usage
- [ ] Balance movement speeds and conversion timing
- [ ] Menu and restart functionality

## Technical Notes
- Use simple collision detection for proximity groups
- Shape tools can be basic colored geometric sprites
- Conversion happens every 0.5 seconds based on current neighbors
- Visual feedback: highlight Moodels about to change mood