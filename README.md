# L-System Explorer

A real-time 3D L-system visualization tool built with [Bevy](https://bevyengine.org/). Explore parametric grammars with PBR materials, physics-based tropism, genetic breeding, and batch export — in the browser or on desktop.

[**Try it in the browser**](https://thejanusstream.github.io/lsystem-explorer/)

## Features

### Grammar Engine
- **Parametric Rules** — Define production rules with parameters, conditions, and stochastic probabilities
- **Context-Sensitive Matching** — Left/right context operators with `#ignore` for skipping turtle symbols
- **Two-Pass Derivation** — Separate growth and finalization (decomposition) phases for cleaner grammar design
- **Async Derivation** — Background thread compilation prevents UI freezing during high-iteration generation

### Rendering
- **Real-time Editing** — Live grammar compilation with debounced auto-update
- **Parallel Transport Framing** — Smooth branch geometry without gimbal lock
- **3 PBR Material Slots** — Base color, emission, roughness, metallic, UV scale, and procedural textures per slot
- **Prop System** — Spawn discrete meshes (leaf, sphere, cone, cylinder, cube) at grammar-defined positions
- **Tropism & Elasticity** — Gravity-influenced growth simulation

### Genetic Breeding (Nursery)
- **Interactive Evolutionary Computation** — 3x3 population grid rendered in 3D world space
- **Champion Selection** — Click individuals to mark as breeding parents; selected plants show translucent highlight panels
- **Mutation & Crossover** — Evolve rules, constants, materials, angles, step sizes, widths, elasticity, and tropism
- **Adjustable Mutation Rate** — Control evolution intensity per generation
- **Preset Injection** — Load any preset into selected champions as a starting point
- **Error Visualization** — Failed derivations shown with red panels and error messages

### Export
- **OBJ** — Wavefront format with per-mesh material references
- **GLB** — Binary glTF 2.0 with full PBR materials
- **Batch Variations** — Generate multiple stochastic variants in one operation with async progress tracking

### Platform
- **Native** — Desktop app with full performance
- **WASM** — Runs in the browser via WebAssembly

## Quick Start

```bash
# Native
cargo run --release

# WASM
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

## Built-in Presets

Presets from Prusinkiewicz & Lindenmayer's *The Algorithmic Beauty of Plants*:

| Preset | ABOP Reference | Highlights |
|--------|---------------|------------|
| Quadratic Koch Island | Fig 1.6 | Fractal space-filling curve |
| Sierpinski Gasket | Fig 1.10b | Two-pass decomposition example |
| Branching Pattern | Fig 1.39 | Parametric binary tree with tapering |
| Monopodial Tree | Fig 2.6 | Spiral phyllotaxis, 3 branch types |
| Sympodial Tree | Fig 2.7 | Sympodial branching architecture |
| Ternary Tree (Gravity) | Fig 2.8 | Tropism and elasticity simulation |
| Ternary Tree (+Props +Materials) | — | Stochastic rules, 3 materials, leaf and sphere props |

## Grammar Syntax

### Directives

| Directive | Description |
|-----------|-------------|
| `#define NAME VALUE` | Define a constant for use in rules |
| `#ignore: SYMBOLS` | Skip symbols during context matching |
| `omega: ...` | Set the axiom (starting state) |
| `pN: PRED : COND -> SUCC` | Production rule with optional condition |

### Turtle Commands

| Symbol | Parameters | Description |
|--------|------------|-------------|
| `F` | `(length)` | Move forward, drawing a branch segment |
| `f` | `(length)` | Move forward without drawing |
| `+` | `(angle)` | Turn left (yaw) |
| `-` | `(angle)` | Turn right (yaw) |
| `&` | `(angle)` | Pitch down |
| `^` | `(angle)` | Pitch up |
| `/` | `(angle)` | Roll clockwise |
| `\` | `(angle)` | Roll counter-clockwise |
| `\|` | | Turn around (180 degrees) |
| `$` | | Roll to vertical (align up with world Y) |
| `[` | | Push state onto stack |
| `]` | | Pop state from stack |
| `!` | `(width)` | Set branch width |

### Material & Prop Commands

| Symbol | Parameters | Description |
|--------|------------|-------------|
| `,` | `(id)` | Switch to material ID (0, 1, or 2) |
| `'` | `(r, g, b)` | Set vertex color (0.0–1.0 per channel) |
| `~` | `(prop_id, scale)` | Spawn a prop at current position |

Prop ID to mesh mapping (configurable in the UI):
- 0 = Leaf, 1 = Sphere, 2 = Cone, 3 = Cylinder, 4 = Cube

### Conditions

| Condition | Description |
|-----------|-------------|
| `*` | Always match (wildcard) |
| `PROB` | Stochastic: match with probability (0.0–1.0) |
| `x > N` | Parameter comparison |
| `x = N` | Parameter equality |

### Context-Sensitive Rules

```
LEFT_CONTEXT < PREDECESSOR > RIGHT_CONTEXT : CONDITION -> SUCCESSORS
```

- **Left context** (`<`): Symbols that must appear immediately before the predecessor.
- **Right context** (`>`): Symbols that must appear immediately after the predecessor.
- Either or both contexts can be omitted for context-free rules.

Use `#ignore` to skip turtle commands during context checks:

```
#ignore: + - & ^ / \ [ ]
```

**Signal propagation example** (acropetal flow):

```
#ignore: + - & ^ / \ F
omega: B(1) A A A A
p1: B(x) < A -> B(x+1)
p2: B(x) -> B(x)
```

After 1 iteration: `B(1) B(2) A A A` — the signal `B` propagates rightward.

### Two-Pass Derivation (Finalization)

Separate **growth** from **decomposition** (ABOP Chapter 1.3):

1. **Growth Phase** — Main grammar rules execute for N iterations, producing abstract symbols.
2. **Finalization Phase** — A second rule set runs once, decomposing abstract symbols into concrete turtle commands.

In the UI, expand the **Finalization (Decomposition)** panel below the grammar editor. Constants from the growth phase carry over.

For presets, use the `/// DECOMPOSITION ///` separator:

```
#define n 5
omega: A(n)
p1: A(x) : x > 0 -> F I(x) [ + A(x-1) ] [ - A(x-1) ]
p2: I(x) -> I(x)
/// DECOMPOSITION ///
p1: I(x) -> F(x*2)
```

## Example Grammars

### Simple Binary Tree

```
omega: A
p1: A -> F [ + A ] [ - A ]
```

### Parametric Tree with Tapering

```
#define wr 0.707
omega: A(100, 10)
p1: A(l, w) -> !(w) F(l) [ &(30) A(l*0.7, w*wr) ] A(l*0.9, w*wr)
```

### PBR Multi-Material Tree

```
#define MAX 5
omega: ,(0) '(0.5, 0.3, 0.2) A(MAX)
p1: A(t) : t > 0 -> !(t*0.1) F(10) [ &(35) ,(1) '(0.2, 0.8, 0.2) B(t-1) ] A(t-1)
p2: B(t) : t > 0 -> F(5) ~(0, 0.5)
```

### Stochastic Branching

```
omega: A(50)
p1: A(s) : 0.5 -> F(s) [ + A(s*0.7) ] [ - A(s*0.7) ]
p2: A(s) : 0.5 -> F(s) [ & A(s*0.7) ]
```

## Camera Controls

- **Middle Mouse + Drag** — Pan
- **Right Mouse + Drag** — Orbit
- **Scroll Wheel** — Zoom

## Architecture

### Split Reactivity
The update loop distinguishes between two independent dirty paths:
- **Geometry Dirty** — Triggered by grammar, iteration, or interpretation changes. Runs async derivation on a background thread, then rebuilds the mesh.
- **Material Dirty** — Triggered by palette edits (color, roughness, metallic, UV scale, texture). Only updates shader parameters — no geometry rebuild.

Tweaking material colors never causes expensive tree regeneration.

## Building

### Requirements
- Rust 1.85+ (Edition 2024)
- For WASM: `wasm32-unknown-unknown` target

### Native
```bash
cargo build --release
cargo run --release
```

### WASM
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### Tests
```bash
cargo test
```

## Dependencies

- [Bevy](https://bevyengine.org/) 0.18 — Game engine and renderer
- [bevy_egui](https://github.com/mvlabat/bevy_egui) — Immediate mode UI
- [bevy_panorbit_camera](https://github.com/Plonq/bevy_panorbit_camera) — Orbit camera controls

### Symbios Ecosystem

L-System Explorer is the reference application for the Symbios crate family:

- [symbios](https://github.com/TheJanusStream/symbios) — L-system parsing and derivation engine
- [symbios-turtle-3d](https://github.com/TheJanusStream/symbios-turtle-3d) — 3D turtle interpreter
- [symbios-genetics](https://github.com/TheJanusStream/symbios-genetics) — Genetic operators for evolution
- [bevy_symbios](https://github.com/TheJanusStream/bevy_symbios) — Bevy integration: mesh generation, PBR materials, and export

## References

- [Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants* (ABOP)](https://algorithmicbotany.org/papers/abop/abop.pdf)
- [L-system — Wikipedia](https://en.wikipedia.org/wiki/L-system)

## License

[Apache License 2.0](LICENSE)
