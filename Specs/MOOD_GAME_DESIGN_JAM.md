# Mood - Simplified Game Design (Game Jam Version)

## Core Concept
Simple top-down game where players use shape tools to influence Moodel movement and achieve target mood ratios.

## Moodel Types & Movement

### 1. Neutral Moodel (Gray) - Uses Moodel.png
- **Movement**: Moderate, steady wandering
- **Speed**: 150 units/second
- **Behavior**: Baseline state, easily influenced by social interactions

### 2. Calm Moodel (Blue) - Uses MoodelCalm.png
- **Movement**: Slow, straight lines with occasional gentle turns
- **Speed**: 100 units/second
- **Behavior**: Stabilizing influence, comforts others

### 3. Happy Moodel (Yellow) - Uses MoodelHappy.png
- **Movement**: Bouncy, energetic with small hops
- **Speed**: 200 units/second
- **Behavior**: Spreads joy to others, energetic movement

### 4. Rage Moodel (Red) - Uses MoodelRage.png
- **Movement**: Fast, aggressive straight lines
- **Speed**: 300 units/second
- **Behavior**: Aggressive and contagious, disrupts peace

### 5. Sad Moodel (Purple) - Uses MoodelSad.png
- **Movement**: Very slow, drifts aimlessly
- **Speed**: 75 units/second
- **Behavior**: Spreads negativity, seeks isolation

## Mood Interaction System
Moodels change based on **social interactions** when they collide with each other. Each mood influences others differently:

### Collision-Based Mood Changes
When two Moodels collide, their moods interact according to these rules:

#### Rage Interactions (Contagious & Aggressive)
- **Rage + Sad** → **Rage + Rage** (bullying spreads anger)
- **Rage + Calm** → **Rage + Sad** (rage disrupts peace)  
- **Rage + Happy** → **Rage + Sad** (rage kills happiness)
- **Rage + Neutral** → **Rage + Sad** (rage affects neutral negatively)
- **Rage + Rage** → **Rage + Rage** (anger reinforces anger)

#### Happy Interactions (Spreads Joy)
- **Happy + Sad** → **Happy + Calm** (happiness cheers up sadness)
- **Happy + Calm** → **Happy + Happy** (happiness spreads to calm)
- **Happy + Neutral** → **Happy + Calm** (happiness lifts neutral)
- **Happy + Happy** → **Happy + Happy** (joy reinforces joy)

#### Calm Interactions (Stabilizing)
- **Calm + Sad** → **Calm + Calm** (calm comforts sadness)
- **Calm + Neutral** → **Calm + Calm** (calm influences neutral)
- **Calm + Calm** → **Calm + Calm** (peace maintains peace)

#### Sad Interactions (Spreads Negativity)
- **Sad + Neutral** → **Sad + Sad** (sadness spreads to neutral)
- **Sad + Sad** → **Sad + Sad** (sadness reinforces sadness)

#### Neutral Interactions (Random Outcomes)
When two neutrals meet, the outcome is randomized to create variety:
- **25% chance**: **Happy + Happy** (spontaneous joy)
- **25% chance**: **Calm + Calm** (finding peace together)  
- **25% chance**: **Sad + Sad** (becoming melancholy)
- **25% chance**: **Neutral + Calm** (mixed outcome)

### Isolation Decay System
When Moodels haven't interacted for 3+ seconds, they naturally decay toward neutral:
- **Rage** → **Calm** (anger cools down when isolated)
- **Happy** → **Calm** (happiness fades without social energy)
- **Sad** → **Neutral** (sadness gradually lifts in isolation)
- **Calm** → **Neutral** (becomes baseline neutral state)
- **Neutral** → **Neutral** (remains stable when isolated)

**Note**: Cyclical mood progression has been disabled. Moods now only change through social interactions and isolation decay.

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

### Core Systems
- [x] 5 Moodel types with sprites and movement patterns (Neutral, Calm, Happy, Rage, Sad)
- [x] Collision-based mood interaction system with social dynamics
- [x] Isolation decay system for natural mood progression
- [x] Physics-based collision detection and bouncing
- [ ] Basic UI showing current mood ratios

### Shape Tools
- [ ] Line tool - click & drag to create barriers
- [ ] Box tool - click & drag to create containment areas
- [ ] Circle tool - click for attraction field

### Game Loop
- [ ] Level system with target ratios
- [ ] Timer and victory conditions
- [ ] 5 tutorial/progression levels

### Polish & Balance
- [ ] Visual feedback for tools and conversions
- [ ] Sound effects for tool usage
- [ ] Balance movement speeds and conversion timing
- [ ] Menu and restart functionality

## Technical Notes
- **Collision System**: Uses Avian2D physics for real-time collision detection
- **Mood Changes**: Instant social interactions on collision contact (not timer-based)
- **Isolation Decay**: Moodels gradually return to neutral when isolated for 3+ seconds
- **Visual Feedback**: Immediate sprite and color changes reflect mood states
- **Physics**: Proper collision physics with bouncing and mass-based interactions
- **Performance**: Event-driven system scales well with multiple entities