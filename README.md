# Matter Compiler (Working Title)

> Turning intent into manufacturable reality.

This project explores a new kind of compiler: one that transforms **high-level specifications of physical machines** into **manufacturable artefacts**.

Instead of compiling software into machine code, the goal is to compile **design intent** into **physical systems**.

The system is designed to bridge the gap between:

- Human intent (natural language or structured specifications)
- AI-assisted design synthesis
- Physical manufacturing constraints
- Real-world fabrication outputs (CAD, CAM, ECAD, G-code, etc.)

The long-term vision is a **robot/machine compiler** that can reason about:

- mechanical structure
- electronics
- control systems
- manufacturing processes
- assembly constraints

...and produce designs that can actually be built.

This repository is an early prototype of that architecture.

## Core Idea

Modern software development works because we have:

`Source Code → Compiler → Machine Code`

For physical systems, the pipeline is fragmented:

`Requirements → CAD → Simulation → Electronics design → Manufacturing planning → Fabrication`

Each step uses different tools and representations.

This project explores a unified pipeline:

`Human Spec → Design SDK → Intermediate Representation (IR) → Compiler Passes → Manufacturing Outputs`

Where the compiler understands both **design intent** and **manufacturing constraints**.

## Key Concepts

### 1. Human Specification

The system begins with a **structured specification** describing a machine or artefact.

This can include:

- task definition
- constraints
- performance goals
- cost targets
- environmental assumptions

Example:

Build a small robotic arm for desk use.
| | |
|---|---|
| Payload | 500g |
| Reach | 300mm |
| Manufacturing | 3D printing |
| Material | PLA |

This spec is translated into the internal system representation.

### 2. Design SDK

The Design SDK provides a **deterministic language for expressing physical systems**.

This acts as the "source code" of machines.

It allows humans or AI to define:

- components
- constraints
- kinematics
- materials
- connections
- electrical systems

Example (conceptual):

```rust
let arm = RobotArm::new()
    .payload(0.5)
    .reach(0.3)
    .actuator(Servo::standard())
    .material(Material::PLA);
```

The SDK feeds the compiler.

### 3. Intermediate Representation (IR)

The IR is the canonical system model.

It represents the machine as a structured graph of:
- bodies
- joints
- actuators
- electronics
- materials
- constraints
- manufacturing assumptions

The IR is machine-friendly but inspectable.

It is versioned and becomes the core artefact of the system.

### 4. Manufacturing Context

Every compilation target assumes a manufacturing environment.

This is analogous to a CPU architecture in software compilation.

Example:

| | |
|---|---|
| **Target** | Home workshop |
| **Printer** | Bambu P1S |
| **Material** | PLA |
| **Tolerance** | ±0.2mm |

The compiler must respect:
- machine limits
- material properties
- achievable tolerances
- available tools

Manufacturing contexts define what designs are actually buildable.

### 5. Compiler Passes

The compiler transforms the IR through multiple passes.

Example passes:

**Validation**
- structural checks
- kinematic feasibility
- load limits

**Optimisation**
- material usage
- structural reinforcement
- part count reduction

**Manufacturability**
- print orientation
- support generation
- tool access

**Assembly**
- fasteners
- cable routing
- part order

Each pass refines the IR.

### 6. Outputs

The compiler produces artefacts required to build the system:
- CAD models
- STL meshes
- G-code
- PCB layouts
- BOM (bill of materials)
- assembly instructions
- calibration procedures

These artefacts are derived from the IR.

## Initial Prototype Scope

To keep the project grounded, the first prototype is intentionally constrained.

Manufacturing Target
| | |
|---|---|
| Machine | Bambu P1S |
| Material | PLA |
| Process | FDM 3D printing |

Assumptions:
- layer height: 0.2mm
- typical tolerance: ±0.2mm
- build volume: 256 × 256 × 256 mm

The compiler will initially target single-process manufacturing.

## Example Design Target

The first example artefact will be a simple fidget toy.

Reasons:
- minimal mechanical complexity
- printable in PLA
- easy to validate
- no electronics required

Example spec:

| | |
|---|---|
| Object | Fidget spinner |
| Diameter | 60mm |
| Material | PLA |
| Manufacturing | FDM printing |

### Current prototype SDK entry point

The first runnable SDK slice lives in `crates/design-sdk`:

- Author/run the example: `rtk cargo run -p design-sdk --example fidget_spinner`
- End-to-end verification: `rtk cargo test -p design-sdk --test fidget_spinner_example`
- Core slice files:
  - `crates/design-sdk/src/lib.rs` - authoring model and IR lowering contract
  - `crates/design-sdk/examples/fidget_spinner.rs` - concrete example design
  - `crates/design-sdk/tests/fidget_spinner_example.rs` - example verification

The example stays within the current prototype limits: a small PLA spinner-like assembly for the Bambu P1S using FDM only, with no downstream CAD or manufacturing generation.

## Repository Structure

```
matter-compiler/
│
├─ README.md
│
├─ Cargo.toml
│
├─ crates/
│  ├─ manufacturing-context/
│  │
│  ├─ design-sdk/
│  │
│  ├─ ir/
│  │
│  ├─ compiler/
│  │
│  └─ examples/
│
└─ docs/
```

- **manufacturing-context** - Defines machine capabilities and constraints.
- **design-sdk** - Human/AI facing API for describing physical systems.
- **ir** - Intermediate representation for machine designs.
- **compiler** - Compilation pipeline and optimisation passes.
- **examples** - Example designs (fidget toy, simple mechanisms).

## Why This Matters

AI is extremely good at navigating large design spaces.

However, AI cannot currently translate intent directly into buildable physical systems.

The missing layer is a machine design compiler that:
- understands physics
- respects manufacturing limits
- integrates mechanical and electronic systems
- produces real artefacts

This repository is an early experiment toward that system.

## Status

Very early prototype.

Goals for the first milestone:
- define manufacturing context model
- define minimal IR
- implement trivial compiler pipeline
- generate a printable object

## Long-Term Vision

Eventually this system could enable:
- AI-generated robotics
- automated factory design
- self-improving machine ecosystems
- programmable manufacturing pipelines

The long arc is simple:

Intent → Machines

## Next steps

1. **`ManufacturingContext` schema** - Done
2. **Minimal IR graph structure**
3. **First Rust crate layout**
4. **A toy fidget-spinner example**

## Repository Conventions

This repository uses International English spelling exclusively in code, documentation, comments, and user-facing text.
