# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a demographic simulation built with the Bevy game engine in Rust. It models an agent-based population with individuals that age, seek partners, form relationships, conceive children, and die. The simulation can run in both interactive mode (with visual display) and headless mode for testing.

## Core Architecture

The project follows Bevy's plugin-based ECS (Entity Component System) architecture with four main modules:

### Core Modules

- **`individual.rs`**: Demographics system managing Individual entities with Demog components (age, sex), aging lifecycle, and adult status transitions
- **`partner.rs`**: Partner-seeking and relationship formation system with FIFO matching algorithm, widow detection, and relationship cleanup
- **`gestation.rs`**: Reproduction system handling conception probability, gestation duration, and birth events
- **`window.rs`**: Visual display system with 2D grid positioning, sprite rendering, movement animations, and camera controls (interactive mode only)

### Key Components

- `Individual`, `Adult`, `Demog` (age, sex)
- `PartnerSeeking`, `Partner`, `Relationship`, `Partners`
- `RemainingGestation`, `Mother`
- `Position`, `Size`, `MovingTowards` (display only)

### System Scheduling

The simulation uses Bevy's `FixedTimestep` for deterministic timing:
- Aging: 1/12 time units (monthly)
- Partner matching: 1/4 time units (quarterly)
- Conception checks: 1/52 time units (weekly)

## Common Development Commands

### Building and Running
```bash
# Interactive mode with visual display
cargo run

# Headless mode for testing/simulation
cargo run --features headless

# Build only
cargo build
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_partner_seeking

# Run tests in headless mode
cargo test --features headless
```

### Development
```bash
# Check for compilation errors without running
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## Key Constants and Configuration

### Demographics (individual.rs)
- `AGING_TIMESTEP`: 1/12 (monthly aging)
- `DEATH_AGE`: 30.0 (accelerated for testing)
- `PARTNER_SEEKING_AGE`: 20.0 (when individuals become adults)

### Partner System (partner.rs)
- `SEEKING_TIMESTEP`: 1/4 (quarterly partner matching)
- Uses FIFO matching algorithm in `AvailableSeekers` resource

### Gestation (gestation.rs)
- `CONCEPTION_TIMESTEP`: 1/52 (weekly conception checks)
- `MIN_CONCEPTION_AGE`: 25.0, `MAX_CONCEPTION_AGE`: 35.0
- `CONCEPTION_RATE`: 0.5, `GESTATION_DURATION`: 40/52 time units

### Display (window.rs)
- `GRID_WIDTH`/`GRID_HEIGHT`: 15x15 simulation grid
- Interactive controls: Enter key spawns new 18-year-old individual
- Color coding: gray (children), blue (males), pink (females)

## Testing Approach

The project uses Bevy's ECS testing framework with manual world setup:
- Tests create `World` instances and `SystemStage` configurations
- Systems are added to stages with proper dependency ordering
- Tests run systems for multiple timesteps to verify state changes
- Uses `approx` crate for floating-point comparisons

## Dependencies

- **bevy**: "0.6.0" - Core game engine and ECS framework
- **bevy_fly_camera**: "0.8.0" - 2D camera controls for interactive mode
- **rand**: "0.7.3" - Random number generation for demographics
- **approx**: "0.5.1" - Floating-point testing utilities