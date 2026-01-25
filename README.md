# L-System Explorer

A real-time 3D L-system visualization tool built with Bevy. Explore parametric grammars with PBR materials, physics-based tropism, and batch export capabilities.

## Features

- **Real-time Editing** - Live grammar compilation with debounced auto-update
- **Multi-Material PBR** - Three editable materials with base color, emission, and roughness
- **Parallel Transport Framing** - Smooth branch geometry without gimbal lock
- **Tropism & Elasticity** - Gravity-influenced growth simulation
- **Prop System** - Spawn discrete meshes (leaves, spheres, cones) at surface IDs
- **Batch Export** - Generate multiple stochastic variations as OBJ files
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
| `@` | `(metallic)` | Set metallic value (0.0-1.0) |
| `#` | `(roughness)` | Set roughness value (0.0-1.0) |

### Prop Commands

| Symbol | Parameters | Description |
|--------|------------|-------------|
| `~` | `(surface_id, scale)` | Spawn a prop at current position |

Props are mapped to mesh types in the UI:
- Surface ID 0 = Leaf (default)
- Surface ID 1 = Sphere
- Surface ID 2 = Cone
- Surface ID 3 = Cylinder/Cube

### Conditions

| Condition | Description |
|-----------|-------------|
| `*` | Always match (wildcard) |
| `PROB` | Stochastic: match with probability (0.0-1.0) |
| `x > N` | Parameter comparison |
| `x = N` | Parameter equality |

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

## UI Controls

### Grammar Panel
- **Presets** - Load example L-systems from ABOP (Algorithmic Beauty of Plants)
- **Code Editor** - Edit grammar with syntax highlighting
- **Defined Constants** - Drag sliders to adjust `#define` values in real-time

### Interpretation
- **Step** - Default forward distance (when `F` has no parameter)
- **Angle** - Default turn angle in degrees
- **Width** - Default branch width
- **Iterations** - Derivation depth

### Physics & Tropism
- **Elasticity** - How much branches bend toward tropism vector (0-1)
- **Tropism Vector** - Direction of gravitational influence

### Material Settings
Each material (0, 1, 2) can be edited independently:
- **Base Color** - Albedo color (tints vertex colors)
- **Emission** - Glow color
- **Glow Strength** - Emission intensity (0-10)
- **Roughness** - Surface smoothness (0=mirror, 1=matte)

### Prop Settings
- **Prop Scale** - Global scale multiplier for all props
- **Surface ID Mappings** - Assign mesh types to surface IDs

### Batch Export
- **Base Name** - Filename prefix for exports
- **Variations** - Number of stochastic variants to generate
- Files are saved to `./exports/` (native) or downloaded (WASM)

## Camera Controls

- **Middle Mouse + Drag** - Orbit camera
- **Right Mouse + Drag** - Pan camera
- **Scroll Wheel** - Zoom in/out

## Materials

The renderer provides three pre-configured materials:

| ID | Name | Default Use |
|----|------|-------------|
| 0 | Primary | Trunk/branches - metallic finish |
| 1 | Energy | Leaves/details - emissive glow |
| 2 | Matte | Structure - diffuse surface |

Switch materials in grammar with `,{id}` command.

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

- [Bevy](https://bevyengine.org/) 0.17 - Game engine
- [bevy_egui](https://github.com/mvlabat/bevy_egui) - Immediate mode UI
- [bevy_panorbit_camera](https://github.com/Plonq/bevy_panorbit_camera) - Camera controls
- [symbios](https://github.com/codewright/symbios) - L-system parsing and derivation
- [symbios-turtle-3d](https://github.com/codewright/symbios-turtle-3d) - 3D turtle interpreter

## References

- Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants* (ABOP)
- [L-system Wikipedia](https://en.wikipedia.org/wiki/L-system)

## License

MIT
