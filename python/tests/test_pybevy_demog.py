#!/usr/bin/env python3
"""Test script for bevy_abm Python bindings"""

import bevy_abm
import polars as pl

print("Testing pybevy_demog Python interface...")
print()

# Test 1: Basic simulation with DataFrame output
print("Test 1: Running simulation with 20 individuals for 5 years")
results = bevy_abm.run_simulation({
    "initial_population": 20,
    "sim_years": 5.0,
    "export_events": False
})
print(f"  Births: {len(results['births'])} events")
print(f"  Deaths: {len(results['deaths'])} events")
print(f"  Partnerships: {len(results['partnerships'])} events")
print(f"  Breakups: {len(results['breakups'])} events")
print(f"  Widowings: {len(results['widowings'])} events")
assert isinstance(results['births'], pl.DataFrame)
assert isinstance(results['deaths'], pl.DataFrame)
print("✓ Test 1 passed")
print()

# Test 2: Verify DataFrame structure and data
print("Test 2: Running simulation with JSON export and checking DataFrame structure")
results = bevy_abm.run_simulation({
    "initial_population": 30,
    "sim_years": 10.0,
    "death_age": 70.0,
    "conception_rate": 0.5,
    "export_events": True
})
print(f"  Births DataFrame shape: {results['births'].shape}")
print(f"  Births columns: {results['births'].columns}")
print(f"  First few births:\n{results['births'].head()}")
assert 'child_entity' in results['births'].columns
assert 'mother_entity' in results['births'].columns
assert 'time' in results['births'].columns
print("✓ Test 2 passed - check births.json and DataFrames")
print()

# Test 3: Custom parameters and analysis
print("Test 3: Running simulation with custom demographic parameters")
results = bevy_abm.run_simulation({
    "initial_population": 25,
    "sim_years": 3.0,
    "death_age": 60.0,
    "min_conception_age": 20.0,
    "max_conception_age": 40.0,
    "conception_rate": 1.0,
    "breakup_rate": 0.2
})
# Example analysis: births with mothers (not initial population)
births_with_mothers = results['births'].filter(pl.col('mother_entity').is_not_null())
print(f"  Total births: {len(results['births'])}")
print(f"  Births with mothers: {len(births_with_mothers)}")
if len(results['partnerships']) > 0:
    print(f"  Average partnership time: {results['partnerships']['time'].mean():.2f}")
print("✓ Test 3 passed")
print()

print("All tests passed!")
