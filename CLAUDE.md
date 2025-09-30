# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a demographic simulation built with the Bevy game engine in Rust. It models an agent-based population with individuals that age, seek partners, form relationships, conceive children, and die. The simulation can run in both interactive mode (with visual display and WASD camera controls) and headless mode for testing with configurable parameters.

## Core Architecture

The project follows Bevy's plugin-based ECS (Entity Component System) architecture with five main modules:

### Core Modules

- **`individual.rs`**: Demographics system managing Individual entities with Demog components (age, sex), aging lifecycle, adult/elder status transitions, and simulation termination
- **`partner.rs`**: Partner-seeking and relationship formation system with FIFO matching algorithm, widow detection with simultaneous death handling, breakup events, and relationship cleanup
- **`gestation.rs`**: Reproduction system handling conception probability, gestation duration, and birth events
- **`window.rs`**: Visual display system with Vec2-based positioning, sprite rendering, movement animations, WASD camera controls, and camera-aware spawning (interactive mode only)
- **`config.rs`**: Configuration management with command-line argument parsing and simulation parameters

### Key Components

- `Individual`, `Adult`, `Elder`, `Demog` (age, sex)
- `PartnerSeeking`, `Partner`, `Relationship`, `Partners`
- `RemainingGestation`, `Mother`
- `Position` (Vec2-based), `Size`, `MovingTowards` (display only)
- `BreakupEvent` for visual movement feedback

### System Scheduling

The simulation uses Bevy's `on_timer` conditions for deterministic timing:
- Aging: 1/12 time units (monthly) - `AGING_TIMESTEP`
- Partner matching: 1/4 time units (quarterly) - `SEEKING_TIMESTEP`
- Conception checks: 1/52 time units (weekly) - `CONCEPTION_TIMESTEP`
- Widow detection runs in PostUpdate to handle entity despawning

## Common Development Commands

### Building and Running
```bash
# Interactive mode with visual display and WASD camera controls
cargo run

# Headless mode for testing/simulation with configurable parameters
cargo run --features headless
cargo run --features headless -- --initial-population 100 --sim-years 10

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
- Default death age: 70.0 years (configurable via UI slider up to 100)
- Partner seeking ages: 20.0-50.0 years (adult phase)
- Supports automatic simulation termination with `--sim-years` argument

### Partner System (partner.rs)
- `SEEKING_TIMESTEP`: 1/4 (quarterly partner matching)
- Uses FIFO matching algorithm in `AvailableSeekers` resource
- Robust widow detection handles simultaneous partner death
- `BreakupEvent` system provides visual feedback when relationships end

### Gestation (gestation.rs)
- `CONCEPTION_TIMESTEP`: 1/52 (weekly conception checks)
- `MIN_CONCEPTION_AGE`: 25.0, `MAX_CONCEPTION_AGE`: 35.0
- `CONCEPTION_RATE`: 0.5, `GESTATION_DURATION`: 40/52 time units

### Display (window.rs)
- `GRID_WIDTH`/`GRID_HEIGHT`: 15x15 simulation grid with Vec2-based positioning
- Interactive controls: WASD camera movement, Enter spawns individual at camera location
- Color coding: light gray (children), blue (males), pink (females), darkened (elders)
- Camera-aware spawning ensures new individuals appear in current view
- Visual breakup behavior with male movement away from partner location

## Testing Approach

The project uses Bevy's ECS testing framework with manual world setup:
- Tests create `World` instances and `Schedule` configurations (Bevy 0.13+)
- Systems are added to schedules with proper dependency ordering
- Tests use deterministic break conditions instead of fixed iteration counts
- Includes comprehensive test for simultaneous partner death scenarios
- Uses `approx` crate for floating-point comparisons

## Dependencies

- **bevy**: "0.13" - Core game engine and ECS framework
- **bevy_egui**: "0.27" - UI framework for interactive parameter controls
- **rand**: "0.8" - Random number generation for demographics
- **approx**: "0.5.1" - Floating-point testing utilities
- **clap**: "4.0" - Command-line argument parsing for headless mode

## CI/CD

GitHub Actions workflow (`.github/workflows/tests.yml`) runs on pushes/PRs to main:
- Builds both interactive and headless modes
- Runs full test suite
- Tests headless simulation example
- Includes Ubuntu system dependencies for Bevy (libasound2-dev, libudev-dev)