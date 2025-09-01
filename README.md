# Smart Road

A real-time traffic simulation built in Rust using SDL2 that demonstrates intelligent intersection management through collision-free vehicle coordination.

## Overview

This project simulates a smart intersection where vehicles approach from four directions (North, South, East, West) and can turn left, right, or go straight. The intersection uses a grid-based reservation system to prevent collisions and optimize traffic flow.

## Features

- **Real-time vehicle simulation** with visual rendering
- **Smart intersection management** using time-space reservation system
- **Dynamic traffic spawning** from all four directions
- **Three vehicle behaviors**: Left turn, Right turn, Straight
- **Adaptive speed control** based on traffic conditions and intersection access
- **Collision detection and prevention**
- **Performance statistics tracking**

## System Architecture

### Core Components

1. **Vehicle (`vehicle.rs`)**: Individual vehicle entities with movement, collision detection, and intersection awareness
2. **Route System (`route.rs`)**: Defines vehicle directions and routes with positioning logic
3. **Intersection Manager (`intersection.rs`)**: Central coordinator handling vehicle spawning, traffic management, and collision prevention
4. **Velocity System (`velocities.rs`)**: Speed management (Slow, Medium, Fast)

### Key Algorithms

#### Grid-Based Reservation System
The intersection is divided into a grid of cells (configurable zone size). Vehicles request time slots for cells they'll occupy during crossing. Conflicts are detected by checking overlapping time reservations.

#### Traffic Management
- **Speed Adaptation**: Vehicles adjust speed based on following distance and intersection permissions
- **Safety Distances**: Dynamic following distances prevent rear-end collisions
- **Turn Coordination**: Different turn types require different cell reservations

## Installation & Setup

### Prerequisites
- Rust (latest stable version)
- SDL2 development libraries
- SDL2_image library

#### Installing SDL2 on Different Platforms

**Ubuntu/Debian:**
```bash
sudo apt-get install libsdl2-dev libsdl2-image-dev
```

**macOS (with Homebrew):**
```bash
brew install sdl2 sdl2_image
```

**Windows:**
- Download SDL2 development libraries from [libsdl.org](https://www.libsdl.org/download-2.0.php)
- Extract and follow SDL2 Rust setup instructions

### Installation Steps

1. **Clone the repository:**
```bash
git clone https://github.com/moseeh/smart-road
cd smart-road-intersection
```

2. **Build and run:**
```bash
cargo build --release
cargo run
```

### Asset Requirements
Create the following directory structure:
```
assets/
├── road-intersection/
│   └── road-intersection.png     # Intersection background image
└── Cars/
    ├── car1.png                  # Vehicle sprites
    ├── car2.png
    ├── car3.png
    ├── car4.png
    └── car5.png
```

## Usage

### Running the Simulation
```bash
cargo run
```

### Controls
- **Arrow Keys**: Spawn vehicles from specific directions
  - `↑` - North (coming from south)
  - `↓` - South (coming from north)  
  - `→` - East (coming from west)
  - `←` - West (coming from east)
- **R** - Spawn vehicle from random direction with random route
- **ESC** - Exit simulation and display final statistics

### Vehicle Behavior
Each spawned vehicle:
- Selects a random route (Left, Right, or Straight)
- Adjusts speed based on traffic ahead
- Requests intersection permission when approaching
- Executes turns at predetermined coordinates
- Exits the simulation when reaching canvas boundaries

## Technical Details

### Coordinate System
- Canvas: 1000x1000 pixels
- Intersection zone: 350-650 pixels (300x300 square)
- Vehicle dimensions: 40x70 pixels (width x height)

### Speed System
- **Fast**: 7 pixels/frame (~420 pixels/second at 60 FPS)
- **Medium**: 5 pixels/frame (~300 pixels/second)
- **Slow**: 3 pixels/frame (~180 pixels/second)

### Lane Configuration
Vehicles spawn in dedicated lanes based on their intended route:
- Each direction has three lanes (one for each turn type)
- Lane positions are precisely calculated in `get_spawn_position()`
- Turn positions defined in `get_turn_position()`

## Statistics Tracking

The simulation tracks and reports:
- Total vehicles that completed their journey
- Maximum and minimum velocities recorded
- Time spent in intersection (max/min)
- Close call incidents
- Real-time active vehicle count

## Smart Features

### Collision Prevention
- **Grid Reservation System**: Prevents spatial conflicts through time-space booking
- **Dynamic Following Distance**: Maintains safe distances between vehicles
- **Turn Coordination**: Different reservation patterns for different turn types

### Traffic Optimization
- **Adaptive Speed Control**: Vehicles slow down for traffic and intersection conflicts
- **Efficient Lane Usage**: Proper lane assignment based on intended route
- **Real-time Conflict Resolution**: Immediate response to traffic conditions

## File Structure

```
src/
├── main.rs           # Main game loop and SDL2 initialization
├── intersection.rs   # Smart intersection management system
├── vehicle.rs        # Vehicle entity and behavior logic
├── route.rs          # Direction and route handling
└── velocities.rs     # Speed enumeration
```

## Performance Considerations

- Frame rate locked to 60 FPS with VSync
- Efficient collision detection using visual bounds
- Memory-efficient grid system with configurable resolution
- Minimal computational overhead per vehicle update

## Future Enhancements

Potential improvements for the simulation:
- Traffic light integration
- Vehicle priority systems (emergency vehicles)
- More complex intersection geometries
- Advanced pathfinding algorithms
- Statistical analysis tools
- Network simulation capabilities

## Troubleshooting

### Common Issues
1. **Asset Loading Errors**: Ensure all PNG files are in correct directories
2. **SDL2 Not Found**: Install SDL2 development packages for your system
3. **Performance Issues**: Reduce grid resolution in `SmartIntersection::new()`

### Debug Output
The simulation provides console output for:
- Vehicle intersection entry/exit times
- Close call detections
- Final statistics summary

---

This simulation demonstrates practical applications of spatial-temporal algorithms in traffic management and provides a foundation for more advanced transportation system modeling.