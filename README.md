A simple test of running demographic simulation with [Bevy Engine](https://bevyengine.org/).

## Running the Simulation

### Interactive Mode
```bash
cargo run
```

**Controls:**
- **WASD** - Move camera around the simulation area
- **Enter** - Spawn a new individual at the current camera location
- **UI Panel** - Adjust simulation parameters with sliders in real-time

### Headless Mode
```bash
cargo run --features headless [OPTIONS]
```

**Options:**
- `-n, --initial-population <NUMBER>` - Starting population size (default: 0)
- `-s, --sim-years <YEARS>` - Simulation duration in years (optional, runs indefinitely if not specified)

**Examples:**
```bash
# Run with 100 initial individuals for 10 simulation years
cargo run --features headless -- --initial-population 100 --sim-years 10

# Run with 50 individuals indefinitely (Ctrl+C to stop)
cargo run --features headless -- -n 50

# Show help for all options
cargo run --features headless -- --help
```

## Simulation Features

The simulation models:
- **Age-based life stages**: Individuals transition from children → adults → elders → death
- **Partner seeking and relationships**: Adults form partnerships within configurable age ranges
- **Conception and birth**: Partnered individuals can conceive and give birth
- **Breakups and widowhood**: Relationships can end through breakups or partner death
- **Visual feedback**: In interactive mode, see individuals move, form relationships, and age with color coding
