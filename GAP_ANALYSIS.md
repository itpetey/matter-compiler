# Gap Analysis: Current Repository vs "Produce an STL Artefact"

## Assumed Desired Outcome

This analysis assumes the desired outcome is:

- take the existing authored design flow
- compile it far enough to emit a concrete STL artefact
- where success means the repository can generate a mesh file that a downstream slicer can consume

That outcome is still beyond the current README milestone, which explicitly says the repository stops at a typed in-memory plan/report and does not yet generate CAD, STL, or G-code.

## Overall Assessment

The repository is not close to producing a real STL artefact yet.

It has a solid front half:

- a constrained manufacturing context model
- a minimal IR
- a deterministic authoring-to-IR lowering path
- a prototype compiler that validates scope and returns a typed manufacturing plan/report

But it is still missing the entire back half needed to turn that plan into printable output:

- geometry representation
- geometry synthesis
- output artefact generation
- file serialization/export
- mesh-oriented verification

In other words, the repository can currently prove that a design is in scope for a PLA-on-P1S prototype flow, but it cannot yet produce an STL that a slicer can consume.

## What Already Exists

### 1. A constrained manufacturing target

`matter-context` defines the supported prototype environment, including:

- Bambu Lab P1S
- PLA
- FDM process
- 256 x 256 x 256 mm build volume
- 0.2 mm nominal layer height
- +/- 0.2 mm achievable tolerance

This is a useful target definition for an eventual STL-producing pipeline.

### 2. A deterministic authoring and lowering path

`matter-sdk` lets the repository author a small spinner-like assembly, validate it, and lower it into IR deterministically.

That means the project already has a stable front door for a future geometry-to-STL backend.

### 3. A prototype compiler and plan/report

`matter-compiler` validates the prototype context, checks the lowered graph shape, checks dimensional limits against the build envelope, and returns a typed plan/report.

This is useful as a preflight stage before real STL generation.

### 4. Conceptual output modelling in IR

`matter-ir::first_prototype_graph()` already introduces `Artifact` nodes and output relationships for:

- CAD
- STL
- G-code

That shows the intended direction clearly. But it is only a conceptual IR helper, not a real generation path.

## Primary Gaps

These are the major blockers between the current repository and an actual STL artefact.

### 1. No geometric representation of parts

This is the largest gap.

The SDK can describe:

- parts
- assemblies
- interfaces
- connections
- named constraints
- simple typed values such as length, text, count, and boolean

But it cannot describe actual shape.

`Part` in `matter-sdk` only carries an ID, material, interfaces, and constraints. `TypedValue` only supports scalar values such as `LengthMm`, `Text`, `Count`, and `Boolean`. There are no geometry primitives, no profiles, no sketches, no solids, no transforms, no topology, and no dimensional relationships rich enough to reconstruct printable geometry.

Practical effect:

- the compiler knows a spinner should be 60 mm wide
- it does not know the geometry that should exist at 60 mm

Without a geometry model, there is nothing to convert into a mesh.

### 2. No synthesis pass from constraints to geometry

Even if the current scalar constraints are enough to express intent, there is no pass that turns that intent into a concrete part definition.

The current compiler pipeline does:

- validate prototype context
- validate lowered graph shape
- validate build envelope
- assemble an in-memory plan/report

It does not:

- derive a body shape
- place interfaces spatially
- compute wall thicknesses
- generate clearances
- derive manufacturable geometry for each part

Practical effect:

- the current compiler stops before the first geometry-producing step

### 3. No runtime artefact generation path

Although `matter-ir` has conceptual `Artifact` nodes, the actual runtime compiler does not generate any artifact objects.

`matter-compiler` returns:

- `PrototypeCompilation`
- `PrototypePlan`
- `PrototypeCompileReport`

None of those types contain:

- CAD payloads
- mesh payloads
- file handles/paths
- byte buffers
- export metadata

Practical effect:

- there is no object in memory today that could be written to `.stl`

### 4. No STL serialization/export

Repository-wide search shows no file export path for printable outputs.

There is no code that:

- creates STL files
- writes mesh data
- exports to disk as part of the compile flow

Practical effect:

- even if geometry existed, the repository currently has no output boundary where an artefact would leave memory and become a printable file

### 5. No mesh-oriented manufacturability validation beyond bounding dimensions

Current validation checks whether length-like constraints fit inside the build envelope, which is useful but shallow.

It does not check STL-relevant concerns such as:

- minimum wall thickness
- overhangs
- bridging
- unsupported spans
- part stability on the bed
- tolerance stack-up for mating parts
- mesh watertightness
- self-intersections
- manifoldness

Practical effect:

- "fits in the printer" is implemented
- "is a sane printable mesh" is not

## Secondary Gaps

These are not the first blockers, but they will matter once geometry/output work begins.

### 1. The public IR and SDK are broader than the supported runtime compiler

The public model already includes concepts that the current compiler rejects, including:

- `Process`
- `Artifact`
- rotational connections
- electrical placeholder interfaces

That makes the repository directionally interesting, but the executable path is narrower than the type surface.

For STL-output work, this means the project needs either:

- a broader compiler backend that can consume these concepts, or
- stricter scoping so the authoring surface matches what the backend can truly handle

### 2. Prototype context validation is incomplete

`available_tools` exists in the manufacturing context schema, but the compiler does not validate it in the prototype-context check.

That is a small correctness gap for the current prototype and will become more important once geometry generation depends on tool-specific behaviour.

### 3. Tests stop at plan/report generation

The test suite is good for the current prototype, but it does not verify:

- generated geometry
- generated STL
- file export
- slicer-consumable output
- geometric/manufacturing invariants on generated artefacts

That is expected given the current implementation, but it highlights how much of the printable-output path is still absent rather than merely untested.

## Smallest Plausible Path To An STL Artefact

If the goal is to reach STL output as quickly as possible, the smallest realistic path is probably:

1. keep the scope to the existing spinner example and the existing PLA-on-P1S target
2. add a minimal geometry model for the current example only
3. add a synthesis pass that turns the spinner constraints into concrete part geometry
4. extend compiler outputs so STL artefacts can exist as first-class results
5. add STL serialization and disk export
6. add geometry-level and golden-file tests for the generated STL

This is materially smaller than the G-code target because it avoids immediate slicer/toolpath integration.

## Recommended Sequencing

The most useful implementation order is:

1. Define the minimum geometry representation needed for the spinner example.
2. Add a compiler pass that derives concrete geometry from current constraints.
3. Extend compiler outputs so artefacts can exist as first-class results, not just plan/report metadata.
4. Add STL serialization and disk export for the example flow.
5. Add geometry-level tests and golden-output tests.
6. Only after that decide whether to integrate slicing/G-code generation for the P1S target.

## Net Conclusion

Relative to the goal of "produce an STL artefact", the repository is still missing the core production backend.

What is implemented today is a strong prototype preflight/compiler front end. What is missing is everything that turns validated design intent into actual geometry and an exported STL file.

The shortest path to the desired outcome is to introduce a minimal geometry-and-export backend for the existing spinner example, with STL as the first real artefact boundary.
