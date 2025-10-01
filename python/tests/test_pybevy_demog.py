#!/usr/bin/env python3
"""Test script for pybevy_demog Python bindings"""

import pybevy_demog

print("Testing pybevy_demog Python interface...")
print()

# Test 1: Basic simulation
print("Test 1: Running simulation with 20 individuals for 5 years")
pybevy_demog.run_simulation({
    "initial_population": 20,
    "sim_years": 5.0,
    "export_events": False
})
print("✓ Test 1 passed")
print()

# Test 2: With JSON export
print("Test 2: Running simulation with JSON export enabled")
pybevy_demog.run_simulation({
    "initial_population": 30,
    "sim_years": 10.0,
    "death_age": 70.0,
    "conception_rate": 0.5,
    "export_events": True
})
print("✓ Test 2 passed - check births.json")
print()

# Test 3: Custom parameters
print("Test 3: Running simulation with custom demographic parameters")
pybevy_demog.run_simulation({
    "initial_population": 25,
    "sim_years": 3.0,
    "death_age": 60.0,
    "min_conception_age": 20.0,
    "max_conception_age": 40.0,
    "conception_rate": 1.0,
    "breakup_rate": 0.2
})
print("✓ Test 3 passed")
print()

print("All tests passed!")
