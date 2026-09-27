# C3.AVcompressor Development & Continuation Guide

**Project:** C3.AVcompressor  
**Binary:** `c3avcompressor`  
**Developer Team:** C3net Development  

---

## 1. Project Directory Structure

```text
C3.AVcompressor/
├── Cargo.toml                  # Rust package manifest & metadata
├── Cargo.lock                  # Dependency lockfile
├── Continue.sh                 # Development resumption script (checks build, tests, NPU & graph)
├── README.md                   # User manual, commands & examples
├── .gitignore                  # Git ignore rules
├── docs/
│   ├── ARCHITECTURE.md         # Detailed hardware & NPU pipeline architecture
│   ├── BENCHMARKS.md           # NPU vs CUDA throughput and power efficiency metrics
│   └── DEV_GUIDE.md            # Developer workflows and dev-pause specification
├── graphify-out/               # Knowledge graph directory
│   ├── graph.json              # Machine-readable AST & dependency graph (155 nodes, 272 edges)
│   └── graph.html              # Interactive visual knowledge graph (open in any browser)
├── src/
│   ├── main.rs                 # CLI entry point, argument parsing & dispatch
│   ├── lib.rs                  # Library entry point for integration tests
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── args.rs             # Clap parser (compress, split, extract, probe, npu-status, benchmark)
│   │   └── commands.rs         # Command handlers with rich terminal output
│   ├── core/
│   │   ├── mod.rs
│   │   ├── probe.rs            # Stream probing & metadata inspection
│   │   ├── splitter.rs         # Lossless stream copy, range cut, and AI scene splitting
│   │   ├── extractor.rs        # Stream demuxing for AAC, AC3, DTS, WAV, MP3, FLAC, Opus
│   │   └── compressor.rs       # Transcoding engine with NPU AI rate control
│   ├── npu/
│   │   ├── mod.rs
│   │   ├── types.rs            # Data structures for NPU devices, plans, and saliency
│   │   ├── detector.rs         # Hardware detection (Apple Silicon ANE 38 TOPS, Intel, AMD)
│   │   └── ai_engine.rs        # Perceptual saliency, dynamic CRF, motion skip heuristics
│   ├── hardware/
│   │   ├── mod.rs
│   │   └── detector.rs         # Hardware video encoders (VideoToolbox on macOS, VAAPI/NVENC on Linux)
│   └── utils/
│       ├── mod.rs
│       ├── time.rs             # High-precision time parsing (HH:MM:SS, compound ms/s/m/h)
│       └── system.rs           # Host CPU, RAM, and platform telemetry
└── tests/
    └── core_tests.rs           # Automated unit and integration test suite
```

---

## 2. Dev-Pause & Continue Workflow

### Triggering "dev-pause"
Whenever you wish to pause development, simply type:
```text
dev-pause
```
The agent will:
1. Run all unit and integration tests (`cargo test`).
2. Build the optimized release binary (`cargo build --release`).
3. Update the knowledge graph via `graphify . --code-only`.
4. Refresh `graphify-out/graph.html` for visual exploration.
5. Synchronize the `memory` MCP graph with current state and milestones.
6. Refresh `Continue.sh` with the latest status.
7. Print a summary of completed milestones and next steps.

### Resuming with `./Continue.sh`
To resume at any later time:
```bash
./Continue.sh
```
This script runs non-interactively, verifies tools and environment, checks that tests pass, displays NPU status, refreshes the knowledge graph, and provides immediate continuity.

---

## 3. Cross-Platform Compilation

### Building on macOS (Apple Silicon & Intel)
```bash
# Native Apple Silicon (ARM64)
cargo build --release

# Intel x86_64 on Apple Silicon (Universal binary support)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

### Building on Linux
```bash
# Native Linux build
cargo build --release

# Cross-compiling for Linux from macOS (using cross)
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
```
