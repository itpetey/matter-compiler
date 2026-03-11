use matter_context::{BuildVolume, ManufacturingContext, Material};
use matter_ir::{Edge, EdgeKind, IrGraph, Node, NodeId, NodeKind, Value};
use matter_sdk::{Design, LoweredDesign};

use crate::{
    CompileError, PrototypeCompilation, PrototypeCompileReport, PrototypeConnectionKind,
    PrototypeConnectionPlan, PrototypeDimensionCheck, PrototypePartPlan, PrototypePass,
    PrototypePlan, PrototypeProfile,
};

/// Compiles an authored SDK design into the current prototype plan/report.
pub fn compile_prototype(design: &Design) -> Result<PrototypeCompilation, CompileError> {
    let lowered = design.lower_to_ir().map_err(CompileError::Lowering)?;
    compile_lowered_prototype(lowered)
}

/// Compiles a lowered SDK design that follows the current prototype IR flow.
pub fn compile_lowered_prototype(
    lowered: LoweredDesign,
) -> Result<PrototypeCompilation, CompileError> {
    let LoweredDesign {
        graph,
        manufacturing,
    } = lowered;

    ensure_prototype_context(&manufacturing)?;
    let analysis = analyse_graph(&graph, &manufacturing)?;

    Ok(PrototypeCompilation {
        design_name: graph
            .node(graph.root())
            .expect("IR graphs always contain their root node")
            .name
            .clone(),
        profile: PrototypeProfile::PlaOnP1sSingleProcess,
        plan: PrototypePlan {
            machine: manufacturing.target.machine.clone(),
            process: manufacturing.target.process,
            material: Material::Pla,
            build_volume: manufacturing.constraints.build_volume,
            nominal_layer_height: manufacturing.constraints.nominal_layer_height,
            achievable_tolerance: manufacturing.constraints.achievable_tolerance,
            parts: analysis.parts,
            connections: analysis.connections,
        },
        report: PrototypeCompileReport {
            executed_passes: vec![
                PrototypePass::ValidatePrototypeContext,
                PrototypePass::ValidateLoweredGraph,
                PrototypePass::ValidateBuildEnvelope,
                PrototypePass::AssembleSingleProcessPlan,
            ],
            validated_part_count: analysis.part_count,
            validated_connection_count: analysis.connection_count,
            validated_dimensions: analysis.validated_dimensions,
        },
        graph,
        manufacturing,
    })
}

struct GraphAnalysis {
    parts: Vec<PrototypePartPlan>,
    connections: Vec<PrototypeConnectionPlan>,
    validated_dimensions: Vec<PrototypeDimensionCheck>,
    part_count: usize,
    connection_count: usize,
}

fn analyse_graph(
    graph: &IrGraph,
    manufacturing: &ManufacturingContext,
) -> Result<GraphAnalysis, CompileError> {
    let build_limit = max_build_dimension(manufacturing.constraints.build_volume);
    let mut part_nodes = Vec::new();
    let mut validated_dimensions = Vec::new();

    for (node_id, node) in graph.nodes() {
        match node.kind {
            NodeKind::System
            | NodeKind::Assembly
            | NodeKind::Part
            | NodeKind::Interface
            | NodeKind::Constraint
            | NodeKind::Material => {}
            _ => {
                return Err(CompileError::UnsupportedNodeKind {
                    kind: node.kind.clone(),
                });
            }
        }

        match node.kind {
            NodeKind::Part => part_nodes.push((*node_id, node.name.clone())),
            NodeKind::Interface => ensure_mechanical_interface(node)?,
            NodeKind::Material => ensure_pla_material(node)?,
            NodeKind::Constraint => {
                if let Some(check) = validate_dimension(node, build_limit)? {
                    validated_dimensions.push(check);
                }
            }
            _ => {}
        }
    }

    let mut part_materials: Vec<(NodeId, String)> = Vec::new();
    let mut connections = Vec::new();

    for edge in graph.edges().values() {
        match edge.kind {
            EdgeKind::Contains | EdgeKind::Constrains => {}
            EdgeKind::UsesMaterial => {
                let source = node_name(graph, edge.source);
                let target = node_name(graph, edge.target);
                if graph.node(edge.source).map(|node| &node.kind) != Some(&NodeKind::Part)
                    || graph.node(edge.target).map(|node| &node.kind) != Some(&NodeKind::Material)
                {
                    return Err(CompileError::InvalidGraphRelationship {
                        edge: edge.kind.clone(),
                        source,
                        target,
                    });
                }
                part_materials.push((edge.source, target));
            }
            EdgeKind::Connects => {
                let source = node_name(graph, edge.source);
                let target = node_name(graph, edge.target);
                if graph.node(edge.source).map(|node| &node.kind) != Some(&NodeKind::Interface)
                    || graph.node(edge.target).map(|node| &node.kind) != Some(&NodeKind::Interface)
                {
                    return Err(CompileError::InvalidGraphRelationship {
                        edge: edge.kind.clone(),
                        source,
                        target,
                    });
                }

                let name = edge_string_attr(graph, edge, "name")?;
                let kind = edge_enum_attr(graph, edge, "connection_kind")?;
                if kind != "fixed" {
                    return Err(CompileError::UnsupportedConnectionKind {
                        connection: name,
                        kind,
                    });
                }

                connections.push(PrototypeConnectionPlan {
                    name,
                    kind: PrototypeConnectionKind::Fixed,
                });
            }
            _ => {
                return Err(CompileError::UnsupportedEdgeKind {
                    kind: edge.kind.clone(),
                });
            }
        }
    }

    let parts = part_nodes
        .into_iter()
        .map(|(part_id, part_name)| {
            let materials = part_materials
                .iter()
                .filter(|(node_id, _)| *node_id == part_id)
                .map(|(_, material)| material)
                .collect::<Vec<_>>();

            if materials.len() != 1 {
                return Err(CompileError::InvalidPartMaterialAssignments {
                    part: part_name,
                    count: materials.len(),
                });
            }

            if materials[0].as_str() != "PLA" {
                return Err(CompileError::UnsupportedMaterial {
                    material: materials[0].clone(),
                });
            }

            Ok(PrototypePartPlan {
                part: part_name,
                material: Material::Pla,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(GraphAnalysis {
        part_count: parts.len(),
        connection_count: connections.len(),
        parts,
        connections,
        validated_dimensions,
    })
}

fn ensure_prototype_context(manufacturing: &ManufacturingContext) -> Result<(), CompileError> {
    let prototype = ManufacturingContext::initial_prototype();

    require_match(
        "target.environment.name",
        &prototype.target.environment.name,
        &manufacturing.target.environment.name,
    )?;
    require_match(
        "target.process",
        &prototype.target.process,
        &manufacturing.target.process,
    )?;
    require_match(
        "target.machine.manufacturer",
        &prototype.target.machine.manufacturer,
        &manufacturing.target.machine.manufacturer,
    )?;
    require_match(
        "target.machine.model",
        &prototype.target.machine.model,
        &manufacturing.target.machine.model,
    )?;
    require_match(
        "constraints.build_volume",
        &prototype.constraints.build_volume,
        &manufacturing.constraints.build_volume,
    )?;
    require_match(
        "constraints.supported_materials",
        &prototype.constraints.supported_materials,
        &manufacturing.constraints.supported_materials,
    )?;
    require_match(
        "constraints.nominal_layer_height",
        &prototype.constraints.nominal_layer_height,
        &manufacturing.constraints.nominal_layer_height,
    )?;
    require_match(
        "constraints.achievable_tolerance",
        &prototype.constraints.achievable_tolerance,
        &manufacturing.constraints.achievable_tolerance,
    )?;

    Ok(())
}

fn ensure_mechanical_interface(node: &Node) -> Result<(), CompileError> {
    let interface_kind = node_enum_attr(node, "interface_kind")?;
    if interface_kind == "mechanical" {
        return Ok(());
    }

    Err(CompileError::UnsupportedInterfaceKind {
        interface: node.name.clone(),
        kind: interface_kind,
    })
}

fn ensure_pla_material(node: &Node) -> Result<(), CompileError> {
    if node.name == "PLA" {
        return Ok(());
    }

    Err(CompileError::UnsupportedMaterial {
        material: node.name.clone(),
    })
}

fn validate_dimension(
    node: &Node,
    build_limit: u16,
) -> Result<Option<PrototypeDimensionCheck>, CompileError> {
    let Some(value) = node.attrs.get("value") else {
        return Ok(None);
    };

    match value {
        Value::Quantity(quantity) if quantity.unit == "mm" => {
            if quantity.value > f64::from(build_limit) {
                return Err(CompileError::ExceedsBuildVolume {
                    name: node.name.clone(),
                    value_mm: quantity.value,
                    limit_mm: build_limit,
                });
            }

            Ok(Some(PrototypeDimensionCheck {
                name: node.name.clone(),
                value_mm: quantity.value,
                build_volume_limit_mm: build_limit,
            }))
        }
        _ => Ok(None),
    }
}

fn max_build_dimension(build_volume: BuildVolume) -> u16 {
    build_volume
        .x_mm
        .max(build_volume.y_mm)
        .max(build_volume.z_mm)
}

fn require_match<T>(field: &'static str, expected: &T, found: &T) -> Result<(), CompileError>
where
    T: PartialEq + std::fmt::Debug,
{
    if expected == found {
        return Ok(());
    }

    Err(CompileError::UnsupportedPrototypeContext {
        field,
        expected: format!("{expected:?}"),
        found: format!("{found:?}"),
    })
}

fn node_enum_attr(node: &Node, attribute: &'static str) -> Result<String, CompileError> {
    match node.attrs.get(attribute) {
        Some(Value::Enum(value)) => Ok(value.clone()),
        _ => Err(CompileError::InvalidGraphAttribute {
            element: node.name.clone(),
            attribute,
            expected: "enum",
        }),
    }
}

fn edge_string_attr(
    graph: &IrGraph,
    edge: &Edge,
    attribute: &'static str,
) -> Result<String, CompileError> {
    match edge.attrs.get(attribute) {
        Some(Value::String(value)) => Ok(value.clone()),
        _ => Err(CompileError::InvalidGraphAttribute {
            element: edge_label(graph, edge),
            attribute,
            expected: "string",
        }),
    }
}

fn edge_enum_attr(
    graph: &IrGraph,
    edge: &Edge,
    attribute: &'static str,
) -> Result<String, CompileError> {
    match edge.attrs.get(attribute) {
        Some(Value::Enum(value)) => Ok(value.clone()),
        _ => Err(CompileError::InvalidGraphAttribute {
            element: edge_label(graph, edge),
            attribute,
            expected: "enum",
        }),
    }
}

fn edge_label(graph: &IrGraph, edge: &Edge) -> String {
    format!(
        "{:?} {} -> {}",
        edge.kind,
        node_name(graph, edge.source),
        node_name(graph, edge.target)
    )
}

fn node_name(graph: &IrGraph, node_id: NodeId) -> String {
    graph
        .node(node_id)
        .map(|node| node.name.clone())
        .unwrap_or_else(|| format!("node-{}", node_id.raw()))
}
