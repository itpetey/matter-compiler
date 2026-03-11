//! Prototype compilation pipeline for the current Matter milestone.

mod artifact;
mod pipeline;
mod spinner_geometry;

use std::{error::Error, fmt};

use matter_context::{
    BuildVolume, LayerHeight, MachineIdentity, ManufacturingContext, Material, ProcessType,
    Tolerance,
};
use matter_ir::{EdgeKind, IrGraph, NodeKind};
use matter_sdk::LoweringError;

pub use artifact::{
    PrototypeArtifact, PrototypeArtifactFormat, PrototypeLengthUnit, PrototypeTriangle,
    PrototypeTriangleMesh, PrototypeVertex,
};
pub use pipeline::{compile_lowered_prototype, compile_prototype};

/// Structured result returned when the prototype compiler accepts an input.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeCompilation {
    /// Name of the root design being compiled.
    pub design_name: String,
    /// Prototype profile accepted by the compiler.
    pub profile: PrototypeProfile,
    /// Canonical lowered IR graph accepted by the compiler.
    pub graph: IrGraph,
    /// Manufacturing context validated against the supported prototype target.
    pub manufacturing: ManufacturingContext,
    /// Deterministic single-process manufacturing plan for the prototype.
    pub plan: PrototypePlan,
    /// First-class printable artefacts produced for the current prototype backend.
    ///
    /// Each artefact carries an STL-oriented triangle mesh boundary so later passes can
    /// serialize the accepted design without widening the compiler API beyond the current
    /// spinner-first scope.
    pub artifacts: Vec<PrototypeArtifact>,
    /// Structured validation and pass report for the compilation.
    pub report: PrototypeCompileReport,
}

impl PrototypeCompilation {
    /// Returns the compiled artefact with the provided stable name.
    pub fn artifact(&self, name: &str) -> Option<&PrototypeArtifact> {
        self.artifacts.iter().find(|artifact| artifact.name == name)
    }

    /// Serializes the named compiled artefact as deterministic ASCII STL content.
    pub fn stl_artifact(&self, name: &str) -> Option<String> {
        self.artifact(name).map(PrototypeArtifact::to_ascii_stl)
    }

    /// Returns the number of nodes in the compiled IR graph.
    pub fn node_count(&self) -> usize {
        self.graph.nodes().len()
    }

    /// Returns the number of edges in the compiled IR graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edges().len()
    }
}

/// Supported prototype profile for this milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeProfile {
    /// PLA, Bambu Lab P1S, and a single FDM manufacturing process.
    PlaOnP1sSingleProcess,
}

/// Deterministic manufacturing plan produced for the supported prototype.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypePlan {
    /// Target machine selected for the prototype.
    pub machine: MachineIdentity,
    /// Manufacturing process selected for the prototype.
    pub process: ProcessType,
    /// Material selected for all compiled parts.
    pub material: Material,
    /// Prototype build volume used during manufacturability checks.
    pub build_volume: BuildVolume,
    /// Nominal layer height used by the prototype target.
    pub nominal_layer_height: LayerHeight,
    /// Achievable tolerance used by the prototype target.
    pub achievable_tolerance: Tolerance,
    /// Part-level manufacturing assignments accepted by the compiler.
    pub parts: Vec<PrototypePartPlan>,
    /// Connection-level manufacturing assignments accepted by the compiler.
    pub connections: Vec<PrototypeConnectionPlan>,
}

/// Deterministic manufacturing assignment for a compiled part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrototypePartPlan {
    /// Authored part identifier retained from the lowered graph.
    pub part: String,
    /// Material assignment accepted for the part.
    pub material: Material,
}

/// Deterministic manufacturing assignment for a compiled connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrototypeConnectionPlan {
    /// Authored connection identifier retained from the lowered graph.
    pub name: String,
    /// Prototype-supported connection kind.
    pub kind: PrototypeConnectionKind,
}

/// Connection kinds accepted by the current prototype pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeConnectionKind {
    /// Fixed mechanical joints within the current spinner-style prototype scope.
    Fixed,
}

/// Structured report describing the prototype pass sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeCompileReport {
    /// Ordered pass sequence executed for the successful compilation.
    pub executed_passes: Vec<PrototypePass>,
    /// Number of part nodes validated for prototype manufacturability.
    pub validated_part_count: usize,
    /// Number of supported connections validated for the prototype.
    pub validated_connection_count: usize,
    /// Dimensional checks performed against the prototype build envelope.
    pub validated_dimensions: Vec<PrototypeDimensionCheck>,
}

/// Deterministic passes executed by the prototype compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypePass {
    /// Confirm the manufacturing context matches the supported prototype target.
    ValidatePrototypeContext,
    /// Confirm the lowered graph remains within the supported prototype IR shape.
    ValidateLoweredGraph,
    /// Confirm dimensional constraints fit within the prototype build envelope.
    ValidateBuildEnvelope,
    /// Build the final single-process manufacturing plan and report.
    AssembleSingleProcessPlan,
    /// Synthesize deterministic printable artefacts for the supported prototype shape.
    SynthesizeArtifacts,
}

/// Successful dimensional validation performed during compilation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeDimensionCheck {
    /// Constraint or requirement name taken from the lowered graph.
    pub name: String,
    /// Checked value in millimetres.
    pub value_mm: f64,
    /// Maximum build-envelope dimension used as the acceptance limit.
    pub build_volume_limit_mm: u16,
}

/// Errors returned when compiling the current prototype design.
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// The authored SDK model could not be lowered into IR.
    Lowering(LoweringError),
    /// The manufacturing context does not match the supported prototype target.
    UnsupportedPrototypeContext {
        /// Path-like field name describing the mismatched context value.
        field: &'static str,
        /// Canonical value expected by the current prototype compiler.
        expected: String,
        /// Value provided by the authored design.
        found: String,
    },
    /// The lowered graph contains a node kind outside the supported prototype flow.
    UnsupportedNodeKind {
        /// Unsupported node kind encountered in the lowered graph.
        kind: NodeKind,
    },
    /// The lowered graph contains an edge kind outside the supported prototype flow.
    UnsupportedEdgeKind {
        /// Unsupported edge kind encountered in the lowered graph.
        kind: EdgeKind,
    },
    /// The lowered graph contains an unsupported interface flavour.
    UnsupportedInterfaceKind {
        /// Lowered interface node name.
        interface: String,
        /// Unsupported interface kind label found in the graph.
        kind: String,
    },
    /// The lowered graph contains an unsupported material assignment.
    UnsupportedMaterial {
        /// Unsupported material label found in the graph.
        material: String,
    },
    /// The lowered graph contains an unsupported connection flavour.
    UnsupportedConnectionKind {
        /// Lowered connection name.
        connection: String,
        /// Unsupported connection kind label found in the graph.
        kind: String,
    },
    /// The validated prototype graph does not match the current spinner-only synthesis path.
    UnsupportedPrototypeShape {
        /// Lowered design name.
        design: String,
        /// Reason the shape fell outside the current geometry synthesis scope.
        reason: String,
    },
    /// A part does not have exactly one prototype material assignment.
    InvalidPartMaterialAssignments {
        /// Lowered part node name.
        part: String,
        /// Number of material edges attached to the part.
        count: usize,
    },
    /// A graph relationship does not match the supported lowered SDK shape.
    InvalidGraphRelationship {
        /// Edge kind on the invalid relationship.
        edge: EdgeKind,
        /// Lowered source node name.
        source: String,
        /// Lowered target node name.
        target: String,
    },
    /// A dimensional constraint exceeds the supported build envelope.
    ExceedsBuildVolume {
        /// Constraint or requirement name that exceeded the envelope.
        name: String,
        /// Requested value in millimetres.
        value_mm: f64,
        /// Supported build-envelope limit in millimetres.
        limit_mm: u16,
    },
    /// A required lowered graph attribute is missing or has the wrong type.
    InvalidGraphAttribute {
        /// Graph element whose attribute could not be interpreted.
        element: String,
        /// Attribute name that failed validation.
        attribute: &'static str,
        /// Attribute type expected by the prototype compiler.
        expected: &'static str,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowering(error) => write!(f, "failed to lower design into IR: {error:?}"),
            Self::UnsupportedPrototypeContext {
                field,
                expected,
                found,
            } => write!(
                f,
                "unsupported prototype context for {field}: expected {expected}, found {found}"
            ),
            Self::UnsupportedNodeKind { kind } => {
                write!(f, "unsupported prototype node kind: {kind:?}")
            }
            Self::UnsupportedEdgeKind { kind } => {
                write!(f, "unsupported prototype edge kind: {kind:?}")
            }
            Self::UnsupportedInterfaceKind { interface, kind } => {
                write!(f, "unsupported interface kind for {interface}: {kind}")
            }
            Self::UnsupportedMaterial { material } => {
                write!(f, "unsupported prototype material: {material}")
            }
            Self::UnsupportedConnectionKind { connection, kind } => {
                write!(f, "unsupported connection kind for {connection}: {kind}")
            }
            Self::UnsupportedPrototypeShape { design, reason } => {
                write!(f, "unsupported prototype shape for {design}: {reason}")
            }
            Self::InvalidPartMaterialAssignments { part, count } => write!(
                f,
                "part {part} must have exactly one prototype material assignment, found {count}"
            ),
            Self::InvalidGraphRelationship {
                edge,
                source,
                target,
            } => write!(
                f,
                "invalid prototype graph relationship {edge:?}: {source} -> {target}"
            ),
            Self::ExceedsBuildVolume {
                name,
                value_mm,
                limit_mm,
            } => write!(
                f,
                "constraint {name} exceeds prototype build volume: {value_mm}mm > {limit_mm}mm"
            ),
            Self::InvalidGraphAttribute {
                element,
                attribute,
                expected,
            } => write!(
                f,
                "invalid graph attribute {attribute} on {element}: expected {expected}"
            ),
        }
    }
}

impl Error for CompileError {}
