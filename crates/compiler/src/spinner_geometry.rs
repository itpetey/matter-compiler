use std::{collections::BTreeMap, f64::consts::PI};

use matter_ir::{Edge, EdgeKind, IrGraph, NodeId, NodeKind, Value};

use crate::{
    CompileError, PrototypeArtifact, PrototypeTriangle, PrototypeTriangleMesh, PrototypeVertex,
};

const BODY_PROFILE_SAMPLES: usize = 36;
const CAP_PROFILE_SAMPLES: usize = 24;
const BODY_HUB_RADIUS_FACTOR: f64 = 0.35;
const CAP_RADIUS_FACTOR: f64 = 1.0 / 6.0;

struct SpinnerGeometryInputs {
    outer_diameter_mm: f64,
    retention_depth_mm: f64,
}

pub(crate) fn synthesize_spinner_artifacts(
    graph: &IrGraph,
) -> Result<Vec<PrototypeArtifact>, CompileError> {
    let geometry = extract_spinner_geometry_inputs(graph)?;
    let body_mesh =
        synthesize_spinner_body_mesh(geometry.outer_diameter_mm, geometry.retention_depth_mm);
    let cap_mesh =
        synthesize_spinner_cap_mesh(geometry.outer_diameter_mm, geometry.retention_depth_mm);

    Ok(vec![
        PrototypeArtifact::stl("body", body_mesh),
        PrototypeArtifact::stl("cap", cap_mesh),
    ])
}

fn extract_spinner_geometry_inputs(graph: &IrGraph) -> Result<SpinnerGeometryInputs, CompileError> {
    let design_name = node_name(graph, graph.root());
    if design_name != "fidget_spinner" {
        return Err(unsupported_shape(
            &design_name,
            "only the existing fidget_spinner synthesis path is supported",
        ));
    }

    let part_nodes = graph
        .nodes()
        .iter()
        .filter(|(_, node)| node.kind == NodeKind::Part)
        .map(|(node_id, node)| (node.name.clone(), *node_id))
        .collect::<BTreeMap<_, _>>();
    let actual_parts = part_nodes.keys().cloned().collect::<Vec<_>>();
    let expected_parts = vec!["body".to_string(), "cap".to_string()];
    if actual_parts != expected_parts {
        return Err(unsupported_shape(
            &design_name,
            &format!("expected printable parts {expected_parts:?}, found {actual_parts:?}"),
        ));
    }

    let connects = graph
        .edges()
        .values()
        .filter(|edge| edge.kind == EdgeKind::Connects)
        .collect::<Vec<_>>();
    if connects.len() != 1 {
        return Err(unsupported_shape(
            &design_name,
            &format!(
                "expected exactly one fixed spinner connection, found {}",
                connects.len()
            ),
        ));
    }

    let connection = connects[0];
    let connection_name = edge_string_attr(graph, connection, "name")?;
    let connection_kind = edge_enum_attr(graph, connection, "connection_kind")?;
    let connection_source = node_name(graph, connection.source);
    let connection_target = node_name(graph, connection.target);
    if connection_name != "cap_join"
        || connection_kind != "fixed"
        || connection_source != "body.cap_socket"
        || connection_target != "cap.cap_spigot"
    {
        return Err(unsupported_shape(
            &design_name,
            &format!(
                "expected cap_join fixed connection between body.cap_socket and cap.cap_spigot, found ({connection_name:?}, {connection_kind:?}, {connection_source:?}, {connection_target:?})"
            ),
        ));
    }

    let root_constraints = named_constraint_values(graph, graph.root())?;
    let body_constraints =
        named_constraint_values(graph, *part_nodes.get("body").expect("body exists"))?;
    let cap_constraints =
        named_constraint_values(graph, *part_nodes.get("cap").expect("cap exists"))?;

    let artefact_class = text_constraint(
        &design_name,
        "artefact_class",
        "artefact_class",
        &root_constraints,
    )?;
    if artefact_class != "fidget_toy" {
        return Err(unsupported_shape(
            &design_name,
            &format!("expected artefact_class = \"fidget_toy\", found {artefact_class:?}"),
        ));
    }

    let outer_diameter_mm = quantity_constraint(
        &design_name,
        "body.outer_diameter",
        "outer_diameter",
        &body_constraints,
    )?;
    let max_diameter_mm = quantity_constraint(
        &design_name,
        "max_diameter",
        "max_diameter",
        &root_constraints,
    )?;
    let retention_depth_mm = quantity_constraint(
        &design_name,
        "cap.retention_depth",
        "retention_depth",
        &cap_constraints,
    )?;

    if !approx_eq(outer_diameter_mm, max_diameter_mm) {
        return Err(unsupported_shape(
            &design_name,
            &format!(
                "expected body.outer_diameter ({outer_diameter_mm}mm) to match max_diameter ({max_diameter_mm}mm)"
            ),
        ));
    }

    Ok(SpinnerGeometryInputs {
        outer_diameter_mm,
        retention_depth_mm,
    })
}

fn synthesize_spinner_body_mesh(
    outer_diameter_mm: f64,
    retention_depth_mm: f64,
) -> PrototypeTriangleMesh {
    let profile = spinner_body_profile(outer_diameter_mm / 2.0);
    extrude_profile(&profile, retention_depth_mm * 2.0)
}

fn synthesize_spinner_cap_mesh(
    outer_diameter_mm: f64,
    retention_depth_mm: f64,
) -> PrototypeTriangleMesh {
    let radius = outer_diameter_mm * CAP_RADIUS_FACTOR;
    let profile = circular_profile(radius, CAP_PROFILE_SAMPLES);
    extrude_profile(&profile, retention_depth_mm)
}

fn spinner_body_profile(outer_radius_mm: f64) -> Vec<(f64, f64)> {
    let hub_radius_mm = outer_radius_mm * BODY_HUB_RADIUS_FACTOR;

    (0..BODY_PROFILE_SAMPLES)
        .map(|index| {
            let angle = 2.0 * PI * index as f64 / BODY_PROFILE_SAMPLES as f64;
            let lobe = (3.0 * angle).cos().max(0.0);
            let radius = hub_radius_mm + (outer_radius_mm - hub_radius_mm) * lobe;
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

fn circular_profile(radius_mm: f64, segments: usize) -> Vec<(f64, f64)> {
    (0..segments)
        .map(|index| {
            let angle = 2.0 * PI * index as f64 / segments as f64;
            (radius_mm * angle.cos(), radius_mm * angle.sin())
        })
        .collect()
}

fn extrude_profile(profile: &[(f64, f64)], height_mm: f64) -> PrototypeTriangleMesh {
    let mut triangles = Vec::with_capacity(profile.len() * 4);
    let bottom_center = vertex(0.0, 0.0, 0.0);
    let top_center = vertex(0.0, 0.0, height_mm);

    for index in 0..profile.len() {
        let next = (index + 1) % profile.len();
        let (x0, y0) = profile[index];
        let (x1, y1) = profile[next];
        let bottom0 = vertex(x0, y0, 0.0);
        let bottom1 = vertex(x1, y1, 0.0);
        let top0 = vertex(x0, y0, height_mm);
        let top1 = vertex(x1, y1, height_mm);

        triangles.push(PrototypeTriangle {
            vertices: [top_center, top0, top1],
        });
        triangles.push(PrototypeTriangle {
            vertices: [bottom_center, bottom1, bottom0],
        });
        triangles.push(PrototypeTriangle {
            vertices: [bottom0, bottom1, top1],
        });
        triangles.push(PrototypeTriangle {
            vertices: [bottom0, top1, top0],
        });
    }

    PrototypeTriangleMesh::new(triangles)
}

fn vertex(x: f64, y: f64, z: f64) -> PrototypeVertex {
    PrototypeVertex {
        x: round_mm(x),
        y: round_mm(y),
        z: round_mm(z),
    }
}

fn round_mm(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn named_constraint_values(
    graph: &IrGraph,
    owner: NodeId,
) -> Result<BTreeMap<String, Value>, CompileError> {
    let mut constraints = BTreeMap::new();

    for edge in graph.edges().values() {
        if edge.kind != EdgeKind::Constrains || edge.source != owner {
            continue;
        }

        let constraint =
            graph
                .node(edge.target)
                .ok_or_else(|| CompileError::InvalidGraphRelationship {
                    edge: edge.kind.clone(),
                    source: node_name(graph, edge.source),
                    target: format!("node-{}", edge.target.raw()),
                })?;
        let value = constraint.attrs.get("value").cloned().ok_or_else(|| {
            CompileError::InvalidGraphAttribute {
                element: constraint.name.clone(),
                attribute: "value",
                expected: "quantity or string",
            }
        })?;
        constraints.insert(constraint.name.clone(), value);
    }

    Ok(constraints)
}

fn text_constraint(
    design_name: &str,
    label: &str,
    key: &'static str,
    constraints: &BTreeMap<String, Value>,
) -> Result<String, CompileError> {
    match constraints.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(CompileError::InvalidGraphAttribute {
            element: design_name.into(),
            attribute: key,
            expected: "string",
        }),
        None => Err(unsupported_shape(
            design_name,
            &format!("missing required spinner constraint {label}"),
        )),
    }
}

fn quantity_constraint(
    design_name: &str,
    label: &str,
    key: &'static str,
    constraints: &BTreeMap<String, Value>,
) -> Result<f64, CompileError> {
    match constraints.get(key) {
        Some(Value::Quantity(quantity)) if quantity.unit == "mm" => Ok(quantity.value),
        Some(_) => Err(CompileError::InvalidGraphAttribute {
            element: design_name.into(),
            attribute: key,
            expected: "millimetre quantity",
        }),
        None => Err(unsupported_shape(
            design_name,
            &format!("missing required spinner constraint {label}"),
        )),
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

fn unsupported_shape(design_name: &str, reason: &str) -> CompileError {
    CompileError::UnsupportedPrototypeShape {
        design: design_name.into(),
        reason: reason.into(),
    }
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn node_name(graph: &IrGraph, node_id: NodeId) -> String {
    graph
        .node(node_id)
        .map(|node| node.name.clone())
        .unwrap_or_else(|| format!("node-{}", node_id.raw()))
}
