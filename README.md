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

Today, the implemented slice is intentionally small: a deterministic in-memory compiler flow for a single-process PLA artefact on a Bambu Lab P1S. The current crates validate and lower an authored `matter-sdk` design, then `matter-compiler` returns a typed plan and pass report. It does **not** yet generate CAD, STL, G-code, PCB, or assembly outputs.

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

Long term, the compiler could produce artefacts required to build the system:
- CAD models
- STL meshes
- G-code
- PCB layouts
- BOM (bill of materials)
- assembly instructions
- calibration procedures

These artefacts would be derived from the IR. The current prototype stops earlier and returns a deterministic in-memory manufacturing plan/report for the supported PLA-on-P1S flow.

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

### Current prototype flow

The first runnable slice lives across `crates/sdk` and `crates/compiler`:

- Author/run the example: `rtk cargo run -p matter-sdk --example fidget_spinner`
- Verify the authored example path: `rtk cargo test -p matter-sdk --test fidget_spinner_example`
- Verify the prototype compiler path: `rtk cargo test -p matter-compiler`
- Core slice files:
  - `crates/sdk/src/lib.rs` - authoring model and IR lowering contract
  - `crates/sdk/examples/fidget_spinner.rs` - concrete example design
  - `crates/sdk/tests/fidget_spinner_example.rs` - end-to-end example verification
  - `crates/compiler/src/lib.rs` - prototype compilation API and typed result types

The verified prototype flow is: author a small PLA spinner-like assembly, lower it deterministically into IR, and compile it into a typed in-memory plan/report for the single supported manufacturing profile.

```rust
use matter_compiler::{PrototypePass, compile_prototype};

let design = fidget_spinner::build_design();
let compiled = compile_prototype(&design)?;

assert_eq!(compiled.plan.parts.len(), 2);
assert_eq!(compiled.report.executed_passes[0], PrototypePass::ValidatePrototypeContext);
```

This stays within the current prototype limits: single-process PLA on a Bambu Lab P1S, deterministic in-memory compilation, and no real CAD/STL/G-code generation.

## Repository Structure

```
matter-compiler/
│
├─ README.md
│
├─ Cargo.toml
│
└─ crates/
   ├─ compiler/
   ├─ context/
   ├─ ir/
   └─ sdk/
```

- **matter-compiler** (`crates/compiler`) - deterministic prototype compilation API and typed plan/report.
- **matter-context** (`crates/context`) - machine capabilities and manufacturing constraints.
- **matter-ir** (`crates/ir`) - intermediate representation for machine designs.
- **matter-sdk** (`crates/sdk`) - authoring API and IR lowering entry point.

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
- implement a deterministic prototype compiler pipeline
- return a typed in-memory manufacturing plan/report
- keep the scope to a single PLA-on-P1S process with no real artefact generation yet

## Long-Term Vision

Eventually this system could enable:
- AI-generated robotics
- automated factory design
- self-improving machine ecosystems
- programmable manufacturing pipelines

The long arc is simple:

Intent → Machines

## Repository Conventions

This repository uses International English spelling exclusively in code, documentation, comments, and user-facing text.
