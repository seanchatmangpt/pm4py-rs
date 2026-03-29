# Python Bindings Installation Guide

Quick setup guide for building and using pm4py-rust Python bindings.

## Quick Start (5 minutes)

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.rustup/env

# Install Python build tools
pip install --upgrade pip maturin
```

### 2. Build the Extension

```bash
cd /path/to/pm4py-rust

# Build in development mode (includes debug symbols)
maturin develop

# OR build release (optimized, smaller binary)
maturin develop --release
```

### 3. Verify Installation

```bash
python -c "from pm4py_rust import EventLog; print('Success!')"
```

## Detailed Build Steps

### Development Build

Best for active development:

```bash
maturin develop
```

This:
- Compiles Rust code (debug mode)
- Creates extension module
- Installs into current Python environment
- Takes ~30-60 seconds on first build
- Rebuilds only changed files on subsequent runs

### Release Build

Best for performance testing:

```bash
maturin develop --release
```

This:
- Compiles with optimizations (-O3)
- LTO enabled
- Takes 2-5 minutes
- ~10x faster than debug build

### Building Wheels for Distribution

```bash
# Single wheel for current Python version
maturin build --release

# Wheels output to: target/wheels/
```

## Troubleshooting Build Issues

### Error: "maturin: command not found"

```bash
pip install maturin
```

### Error: "Could not compile pm4py"

Check Rust version (must be 1.70+):
```bash
rustc --version
```

Update if needed:
```bash
rustup update
```

### Error: "No Rust compiler found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.rustup/env
```

### Error: Python version mismatch

Ensure you're using consistent Python:
```bash
which python
python --version
```

If using virtualenv:
```bash
source /path/to/venv/bin/activate
maturin develop
```

### OSX M1/M2 (Apple Silicon)

If you get architecture errors:
```bash
# Ensure Rust supports native architecture
rustup target add aarch64-apple-darwin
maturin develop
```

### Windows Build Issues

Use Microsoft Visual C++ 14.0:
```bash
# Download from Visual Studio website or use
pip install windows-curses  # if needed
maturin develop
```

## Testing Installation

### Quick Test

```python
from pm4py_rust import EventLog, AlphaMiner

# Create simple log
log = EventLog()
trace = log.add_trace("case_1")
trace.add_event("A", "2024-01-01T00:00:00Z")
trace.add_event("B", "2024-01-01T01:00:00Z")

# Mine
miner = AlphaMiner()
net = miner.apply(log)

print(f"Petri Net: {net.places_count()} places, {net.transitions_count()} transitions")
```

### Run Full Test Suite

```bash
# Build first
maturin develop

# Run Python integration tests
pytest tests/test_python_bindings.py -v

# Run Rust tests with Python feature
cargo test --features python
```

## Virtual Environment Setup

Recommended for clean development:

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install build tools
pip install --upgrade pip maturin pytest

# Build pm4py-rust
cd /path/to/pm4py-rust
maturin develop

# Now use pm4py_rust in this environment
python -c "from pm4py_rust import EventLog; print('Ready!')"
```

## Using pm4py-rust in Your Project

### In Python Code

```python
from pm4py_rust import (
    EventLog, Event, Trace,
    AlphaMiner, InductiveMiner, HeuristicMiner,
    FootprintsConformanceChecker,
    LogStatistics,
    PetriNet, ProcessTree
)

# Your code here
```

### In requirements.txt (after distribution)

```
pm4py-rust>=0.3.0
```

### In setup.py

```python
install_requires=[
    'pm4py-rust>=0.3.0',
],
```

## Performance Tips

### Maximum Performance

```bash
# Build optimized extension
maturin develop --release
```

Then use in Python:
```python
# No special code needed - it's automatically optimized
```

### Measuring Performance

```python
import time
from pm4py_rust import EventLog, AlphaMiner

# Create test log
log = EventLog()
for i in range(1000):
    trace = log.add_trace(f"case_{i}")
    trace.add_event("A", f"2024-01-01T{i%24:02d}:00:00Z")
    trace.add_event("B", f"2024-01-01T{i%24:02d}:01:00Z")

# Time algorithm
miner = AlphaMiner()
start = time.time()
net = miner.apply(log)
elapsed = time.time() - start

print(f"AlphaMiner: {elapsed*1000:.1f}ms for {len(log)} traces")
```

## Updating to New Versions

When pm4py-rust is updated:

```bash
# Get latest code
git pull origin main

# Rebuild extension
maturin develop --release

# Verify
python -c "import pm4py_rust; print(pm4py_rust.__version__)"
```

## Uninstalling

```bash
# Remove Python package
pip uninstall pm4py_rust

# Clean build artifacts (optional)
cargo clean
```

## Useful Commands

```bash
# List installed version
pip show pm4py_rust

# Check available commands
maturin --help

# Build with verbose output
maturin develop --verbose

# Check for compilation warnings
cargo clippy --features python

# Format code (Rust)
cargo fmt
```

## Common Workflows

### Daily Development

```bash
# Terminal 1: Build on file changes
cargo watch -x "build --features python"

# Terminal 2: Run code
python my_script.py
```

### Testing Release Build

```bash
maturin develop --release
pytest tests/test_python_bindings.py
```

### Profiling Performance

```bash
# Run code with profiler
python -m cProfile -s cumtime my_script.py
```

## Need Help?

- GitHub Issues: https://github.com/seanchatmangpt/pm4py-rust/issues
- Documentation: See `docs/PYTHON_BINDINGS.md`
- Build System: https://github.com/PyO3/maturin
