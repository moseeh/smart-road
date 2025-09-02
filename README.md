# Smart Road Intersection

A real-time autonomous vehicle traffic simulation built in Rust using SDL2 that implements intelligent intersection management without traffic lights.

## Project Overview

This project simulates a smart intersection where autonomous vehicles approach from four directions and can turn left, right, or go straight. The system uses a grid-based time-space reservation algorithm to prevent collisions and optimize traffic flow without traditional traffic control mechanisms.

## Implementation Features

### Core Functionality
- **Collision-free intersection management** using time-space cell reservations
- **Real-time vehicle physics** with three distinct velocity levels (Slow/Medium/Fast/Stopped)
- **Dynamic speed adaptation** based on traffic conditions and intersection permissions
- **Safety distance enforcement** between vehicles to prevent collisions
- **Animated vehicle movement** with proper rotation during turns
- **Comprehensive statistics tracking** for performance analysis

### Vehicle Behavior
- **Route adherence**: Vehicles follow predetermined lanes (left turn, straight, right turn)
- **Adaptive speed control**: Automatic velocity adjustment based on traffic and intersection status
- **Turn animation**: Vehicles rotate correctly when changing direction
- **Safety distance**: Maintains configurable following distances

### Intersection Management
- **Grid-based reservation system**: 10x10 pixel cells with time-slot booking
- **Two-path collision detection**: Separate handling for straight and turning vehicles  
- **Dynamic permission system**: Real-time intersection access control
- **Close call detection**: Monitors safety violations between vehicles

## System Architecture

```
src/
├── main.rs           # Game loop, SDL2 initialization, input handling
├── intersection.rs   # Smart intersection management and collision prevention
├── vehicle.rs        # Vehicle physics, movement, and collision detection
├── route.rs          # Direction and route positioning logic
├── stats.rs          # Statistics display with animated background
└── velocities.rs     # Speed enumeration definitions
```

## Installation Requirements

### Dependencies
- Rust (latest stable version)
- SDL2 development libraries
- SDL2_image library
- SDL2_ttf library (for statistics display)

### Platform-Specific Setup

**Ubuntu/Debian:**
```bash
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
```

**macOS (Homebrew):**
```bash
brew install sdl2 sdl2_image sdl2_ttf
```

**Windows:**
Download SDL2, SDL2_image, and SDL2_ttf development libraries from libsdl.org

### Required Assets
Create this directory structure:
```
assets/
├── road-intersection/
│   └── road-intersection.png     # Intersection background image
├── Cars/
│   ├── car1.png                  # Vehicle sprites (40x70 pixels)
│   ├── car2.png
│   ├── car3.png
│   ├── car4.png
│   └── car5.png
└── fonts/
    └── Orbitron-VariableFont_wght.ttf  # Font for statistics display
```

## Usage

### Running the Simulation
```bash
git clone https://github.com/moseeh/smart-road
cd smart-road
cargo run
```

### Controls
- **Arrow Keys**: Spawn vehicles from specific directions
  - Up Arrow: Generate vehicle from south to north
  - Down Arrow: Generate vehicle from north to south  
  - Right Arrow: Generate vehicle from west to east
  - Left Arrow: Generate vehicle from east to west
- **R**: Continuously generate random vehicles
- **S**: Stop continuously spawninng random vehicles
- **ESC**: Exit simulation and display statistics

### Vehicle Generation Rules
- Vehicles spawn with random routes (left/straight/right)
- Anti-spam protection prevents vehicles from spawning on top of each other
- Each vehicle gets a unique ID and texture variant

## Technical Specifications

### Coordinate System
- Canvas dimensions: 1000x1000 pixels
- Intersection zone: 350-650 pixels (300x300 square)
- Vehicle size: 40x70 pixels
- Grid resolution: 10x10 pixel cells for collision detection

### Physics Implementation
- **Velocity system**: 3.0, 5.0, 7.0 pixels/frame (180, 300, 420 pixels/second at 60 FPS)
- **Time calculation**: Based on distance/velocity with frame rate conversion
- **Safety distance**: Configurable following distance (default: 50 pixels + vehicle length)
- **Turn mechanics**: Vehicles execute turns at predetermined coordinates with rotation

### Lane Configuration
Each direction has three dedicated lanes:
- **North**: x-coordinates 500 (left), 550 (straight), 600 (right)
- **South**: x-coordinates 450 (left), 400 (straight), 350 (right)  
- **East**: y-coordinates 500 (left), 550 (straight), 600 (right)
- **West**: y-coordinates 450 (left), 400 (straight), 350 (right)

## Smart Intersection Algorithm

### Time-Space Reservation System
1. **Path calculation**: Pre-computed cell sequences for each direction/route combination
2. **Time slot booking**: Vehicles reserve cells for specific time intervals
3. **Conflict detection**: Prevents overlapping reservations in same cells
4. **Dynamic speed adjustment**: Reduces speed when conflicts detected
5. **Progressive release**: Cells released as vehicles pass through them

### Collision Prevention Strategies
- **Spatial separation**: Grid-based cell reservation prevents same-space conflicts
- **Temporal coordination**: Time-based bookings prevent timing conflicts  
- **Safety buffers**: Following distance requirements prevent rear-end collisions
- **Turn coordination**: Special handling for vehicles changing direction

## Statistics Tracking

The system monitors and reports:
- **Total vehicles passed**: Count of vehicles completing intersection traversal
- **Velocity statistics**: Maximum and minimum speeds recorded across all vehicles
- **Intersection timing**: Maximum and minimum time spent in intersection area
- **Close calls**: Safety distance violations between vehicles
- **Active vehicle count**: Real-time count of vehicles in simulation

Statistics display features:
- Animated car background during statistics screen
- Color-coded text (white labels, yellow values, cyan highlights)
- Orbitron font for futuristic appearance
- Precise label alignment for professional presentation

## Performance Characteristics

- **Frame rate**: Locked at 60 FPS with VSync
- **Memory efficiency**: Reusable vehicle textures, efficient grid storage
- **Computational complexity**: O(n) vehicle updates, O(1) cell access
- **Scalability**: Configurable grid resolution for performance tuning

## Algorithm Advantages

### Compared to Traffic Lights
- **No wait times**: Vehicles proceed when intersection is clear
- **Dynamic adaptation**: Real-time response to traffic patterns
- **Higher throughput**: No fixed timing constraints

### Compared to Human-Driven Systems  
- **Perfect coordination**: No human error or reaction delays
- **Optimal spacing**: Precise safety distance maintenance
- **Predictable behavior**: Deterministic movement patterns

## Build and Development

### Compilation
```bash
cargo build --release    # Optimized build
cargo run                # Development run
cargo test               # Run tests (if implemented)
```

### Code Organization
- **Modular design**: Separate concerns across multiple files
- **Type safety**: Strong typing with Rust's ownership system
- **Error handling**: Proper Result types for fallible operations
- **Performance**: Zero-cost abstractions and efficient algorithms

## Known Limitations

- **Single intersection**: Only handles one intersection type
- **Fixed lanes**: No lane changing or route deviation
- **Deterministic spawning**: Limited randomization in vehicle generation
- **Static assets**: Requires pre-loaded image and font files

## Future Development Possibilities

- Multiple intersection types (T-junctions, roundabouts)
- Variable speed limits and acceleration/deceleration physics
- Emergency vehicle prioritization
- Network of connected intersections
- Machine learning optimization
- Real-world data integration

---

This simulation demonstrates practical applications of autonomous vehicle coordination algorithms and provides a foundation for advanced traffic management system research.