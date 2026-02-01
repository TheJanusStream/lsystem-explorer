# L-System Explorer

A real-time 3D L-system visualization tool built with Bevy. Explore parametric grammars with PBR materials, physics-based tropism, and batch export capabilities.

## Features

- **Real-time Editing** - Live grammar compilation with debounced auto-update
- **Async Derivation** - Background thread compilation prevents UI freezing during high-iteration generation
- **Two-Pass Derivation** - Separate growth and finalization phases for cleaner grammar design
- **Material Palette** - Three editable PBR materials with base color, emission, roughness, metallic, UV scale, and procedural textures
- **Parallel Transport Framing** - Smooth branch geometry without gimbal lock
- **Tropism & Elasticity** - Gravity-influenced growth simulation
- **Prop System** - Spawn discrete meshes (leaves, spheres, cones) at prop IDs
- **Batch Export** - Generate multiple stochastic variations as OBJ or GLB (binary glTF) files
- **WASM Support** - Runs in the browser via WebAssembly

## Quick Start

```bash
# Native
cargo run --release

# WASM (requires wasm-pack)
cargo build --target wasm32-unknown-unknown --release
```

## Grammar Syntax

### Directives

| Directive | Description |
|-----------|-------------|
| `#define NAME VALUE` | Define a constant for use in rules |
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

### PBR Material Commands

| Symbol | Parameters | Description |
|--------|------------|-------------|
| `,` | `(id)` | Switch to material ID (0, 1, or 2) |
| `'` | `(r, g, b)` | Set vertex color (0.0-1.0 per channel) |

### Prop Commands

| Symbol | Parameters | Description |
|--------|------------|-------------|
| `~` | `(prop_id, scale)` | Spawn a prop at current position |

Props are mapped to mesh types in the UI:
- Prop ID 0 = Leaf (default)
- Prop ID 1 = Sphere
- Prop ID 2 = Cone
- Prop ID 3 = Cylinder/Cube

### Conditions

| Condition | Description |
|-----------|-------------|
| `*` | Always match (wildcard) |
| `PROB` | Stochastic: match with probability (0.0-1.0) |
| `x > N` | Parameter comparison |
| `x = N` | Parameter equality |

### Context-Sensitive Rules

Context-sensitive L-systems (also known as 2L-systems or IL-systems) allow production rules to match based on neighboring symbols, not just the predecessor itself. This enables signal propagation, acropetal/basipetal information flow, and interaction between branches.

#### Syntax

```
LEFT_CONTEXT < PREDECESSOR > RIGHT_CONTEXT : CONDITION -> SUCCESSORS
```

- **Left context** (`<`): Symbols that must appear immediately before the predecessor in the current string. Multiple context symbols are read left-to-right.
- **Right context** (`>`): Symbols that must appear immediately after the predecessor. Multiple context symbols are read left-to-right.
- Either or both contexts can be omitted for context-free rules.

#### The `#ignore` Directive

When checking context, turtle commands like `+`, `-`, `&`, `/`, `[`, `]` would break adjacency between biological symbols. The `#ignore` directive tells the matching engine to skip specified symbols during context checks:

```
#ignore: + - & ^ / \ [ ]
```

This means `A + B` still counts as "A is left context of B" because `+` is ignored during matching.

#### Examples

**Signal propagation** (acropetal flow from base to tip):

```
#ignore: + - & ^ / \ F
omega: B(1) A A A A
p1: B(x) < A -> B(x+1)
p2: B(x) -> B(x)
```

After 1 iteration: `B(1) B(2) A A A` — the signal `B` propagates rightward, carrying an incrementing parameter.

**Bidirectional context** (both left and right neighbors must match):

```
#ignore: + -
omega: A C B A
p1: A < C > B -> D
```

`C` is only replaced by `D` when preceded by `A` and followed by `B`.

**Acropetal signal in a branching structure** (ABOP Chapter 1.9):

```
#define D 1
#ignore: + - F
omega: F A F A F A
p1: A < A > A : * -> F A
p2: A < A : * -> A
```

Signals propagate along chains while respecting branch topology — `[` and `]` delimit branches, so context matching does not cross branch boundaries unless those symbols are ignored.

### Two-Pass Derivation (Finalization)

L-System Explorer supports dual-stage grammar execution, commonly used to separate **growth** from **decomposition**. This technique is described in ABOP Chapter 1.3 for modeling plant development.

#### Concept

1. **Growth Phase**: The main grammar rules execute for the specified number of iterations, producing an abstract structural description (e.g., apex symbols, internodes).
2. **Finalization Phase**: A second set of rules runs once, decomposing abstract symbols into concrete turtle commands for rendering.

This separation keeps the growth grammar clean and allows different graphical interpretations of the same structure.

#### Usage

In the UI, expand the **Finalization (Decomposition)** panel below the main Grammar editor. Rules entered there execute after the growth phase completes. Constants defined with `#define` in the growth phase remain available during finalization.

For presets, use the separator `/// DECOMPOSITION ///` to include finalization rules:

```
#define n 5
omega: A(n)
p1: A(x) : x > 0 -> F I(x) [ + A(x-1) ] [ - A(x-1) ]
p2: I(x) -> I(x)
/// DECOMPOSITION ///
p1: I(x) -> F(x*2)
```

#### Example: Abstract to Concrete

**Growth grammar** (main editor):
```
#define LEN 10
omega: A
p1: A -> F [ + A ] F [ - A ] A
p2: F -> S F
```

**Finalization rules** (decomposition editor):
```
p1: S -> F(LEN*0.5)
```

Here, `S` is an abstract "segment" symbol created during growth. The finalization pass converts each `S` to a concrete `F(5)` movement command. The `LEN` constant from the growth phase is accessible in finalization.

#### Behavior Details

- Growth rules execute for N iterations (set by the Iterations slider)
- After growth completes, all growth rules are cleared
- Finalization rules are parsed (constants are preserved)
- A single derivation pass executes the finalization rules
- Any `omega:` lines in finalization are ignored (the growth result is used)

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

- **Middle Mouse + Drag** - Pan camera
- **Right Mouse + Drag** - Orbit camera
- **Scroll Wheel** - Zoom in/out

## Architecture

### Split Reactivity
The update loop distinguishes between two independent dirty paths:
- **Geometry Dirty** - Triggered by grammar, iteration, or interpretation changes. Runs async derivation on a background thread, then rebuilds the mesh.
- **Material Dirty** - Triggered by palette edits (color, roughness, metallic, UV scale, texture). Only updates shader parameters, no geometry rebuild.

This separation means tweaking material colors never causes expensive tree regeneration.

## Building

### Requirements
- Rust 1.85+ (Edition 2024)
- For WASM: `wasm32-unknown-unknown` target

### Native Build
```bash
cargo build --release
cargo run --release
```

### WASM Build
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

## Dependencies

- [Bevy](https://bevyengine.org/) - Game engine
- [bevy_egui](https://github.com/mvlabat/bevy_egui) - Immediate mode UI
- [bevy_panorbit_camera](https://github.com/Plonq/bevy_panorbit_camera) - Camera controls

### The crates L-System Explorer is meant to showcase

- [symbios](https://github.com/TheJanusStream/symbios) - L-system parsing and derivation
- [symbios-turtle-3d](https://github.com/TheJanusStream/symbios-turtle-3d) - 3D turtle interpreter
- [bevy_symbios](https://github.com/TheJanusStream/bevy_symbios) - Bevy mesh generation, materials, and export

## References

- [Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants* (ABOP)](https://algorithmicbotany.org/papers/abop/abop.pdf)
- [L-system Wikipedia](https://en.wikipedia.org/wiki/L-system)
