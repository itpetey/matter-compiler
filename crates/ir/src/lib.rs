use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const IR_SCHEMA_VERSION: u32 = 1;

pub type AttrMap = BTreeMap<String, Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(u64);

impl EdgeId {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    System,
    Assembly,
    Part,
    Interface,
    Constraint,
    Material,
    Process,
    Artifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Contains,
    Connects,
    Constrains,
    UsesMaterial,
    TargetsProcess,
    Produces,
    DerivedFrom,
    Satisfies,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefinementState {
    Abstract,
    Selected,
    Realized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    InputSpec,
    Sdk,
    Pass { name: String },
    Imported { source: String },
}

impl Origin {
    pub fn pass(name: impl Into<String>) -> Self {
        Self::Pass { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Enum(String),
    Quantity(Quantity),
    Range(Range),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub name: String,
    pub state: RefinementState,
    pub attrs: AttrMap,
    pub origin: Origin,
}

impl Node {
    pub fn new(kind: NodeKind, name: impl Into<String>, origin: Origin) -> Self {
        Self {
            kind,
            name: name.into(),
            state: RefinementState::Abstract,
            attrs: AttrMap::new(),
            origin,
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, value: Value) -> Self {
        self.attrs.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub kind: EdgeKind,
    pub source: NodeId,
    pub target: NodeId,
    pub attrs: AttrMap,
    pub origin: Origin,
}

impl Edge {
    pub fn new(kind: EdgeKind, source: NodeId, target: NodeId, origin: Origin) -> Self {
        Self {
            kind,
            source,
            target,
            attrs: AttrMap::new(),
            origin,
        }
    }

    pub fn with_attr(mut self, key: impl Into<String>, value: Value) -> Self {
        self.attrs.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    MissingNode(NodeId),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(node_id) => write!(f, "missing node {}", node_id.raw()),
        }
    }
}

impl Error for GraphError {}

#[derive(Debug, Clone, PartialEq)]
pub struct IrGraph {
    version: u32,
    root: NodeId,
    nodes: BTreeMap<NodeId, Node>,
    edges: BTreeMap<EdgeId, Edge>,
    next_node_id: u64,
    next_edge_id: u64,
    metadata: AttrMap,
}

impl IrGraph {
    pub fn new(root_kind: NodeKind, root_name: impl Into<String>, origin: Origin) -> Self {
        let root = NodeId(0);
        let mut nodes = BTreeMap::new();
        nodes.insert(root, Node::new(root_kind, root_name, origin));

        Self {
            version: IR_SCHEMA_VERSION,
            root,
            nodes,
            edges: BTreeMap::new(),
            next_node_id: 1,
            next_edge_id: 0,
            metadata: AttrMap::new(),
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn metadata(&self) -> &AttrMap {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut AttrMap {
        &mut self.metadata
    }

    pub fn nodes(&self) -> &BTreeMap<NodeId, Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &BTreeMap<EdgeId, Edge> {
        &self.edges
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, edge: Edge) -> Result<EdgeId, GraphError> {
        if !self.nodes.contains_key(&edge.source) {
            return Err(GraphError::MissingNode(edge.source));
        }

        if !self.nodes.contains_key(&edge.target) {
            return Err(GraphError::MissingNode(edge.target));
        }

        let id = EdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.insert(id, edge);
        Ok(id)
    }
}

pub fn first_prototype_graph() -> Result<IrGraph, GraphError> {
    let mut graph = IrGraph::new(NodeKind::System, "fidget_spinner", Origin::InputSpec);
    graph.metadata_mut().insert(
        "prototype".into(),
        Value::String("README first prototype".into()),
    );

    let root = graph.root();

    let design_target = graph.add_node(
        Node::new(NodeKind::Constraint, "design_target", Origin::InputSpec)
            .with_attr("object", Value::String("fidget_spinner".into()))
            .with_attr("category", Value::Enum("fidget_toy".into()))
            .with_attr(
                "diameter",
                Value::Quantity(Quantity {
                    value: 60.0,
                    unit: "mm".into(),
                }),
            )
            .with_attr("electronics_required", Value::Bool(false)),
    );

    let mut body = Node::new(NodeKind::Part, "body", Origin::pass("synthesis"))
        .with_attr("printable_as_single_piece", Value::Bool(true));
    body.state = RefinementState::Realized;
    let body = graph.add_node(body);

    let mut material = Node::new(NodeKind::Material, "PLA", Origin::pass("manufacturability"))
        .with_attr("form", Value::Enum("filament".into()));
    material.state = RefinementState::Selected;
    let material = graph.add_node(material);

    let mut process = Node::new(
        NodeKind::Process,
        "fdm_printing",
        Origin::pass("manufacturability"),
    )
    .with_attr("machine", Value::String("Bambu P1S".into()))
    .with_attr("manufacturing_mode", Value::Enum("single_process".into()));
    process.state = RefinementState::Selected;
    let process = graph.add_node(process);

    let layer_height = graph.add_node(
        Node::new(
            NodeKind::Constraint,
            "layer_height",
            Origin::pass("manufacturability"),
        )
        .with_attr(
            "target",
            Value::Quantity(Quantity {
                value: 0.2,
                unit: "mm".into(),
            }),
        ),
    );

    let tolerance = graph.add_node(
        Node::new(
            NodeKind::Constraint,
            "tolerance",
            Origin::pass("manufacturability"),
        )
        .with_attr(
            "allowed",
            Value::Range(Range {
                min: -0.2,
                max: 0.2,
                unit: "mm".into(),
            }),
        ),
    );

    let build_volume = graph.add_node(
        Node::new(
            NodeKind::Constraint,
            "build_volume",
            Origin::pass("manufacturability"),
        )
        .with_attr(
            "x_max",
            Value::Quantity(Quantity {
                value: 256.0,
                unit: "mm".into(),
            }),
        )
        .with_attr(
            "y_max",
            Value::Quantity(Quantity {
                value: 256.0,
                unit: "mm".into(),
            }),
        )
        .with_attr(
            "z_max",
            Value::Quantity(Quantity {
                value: 256.0,
                unit: "mm".into(),
            }),
        ),
    );

    let mut cad_model = Node::new(NodeKind::Artifact, "cad_model", Origin::pass("output"))
        .with_attr("format", Value::Enum("cad".into()));
    cad_model.state = RefinementState::Realized;
    let cad_model = graph.add_node(cad_model);

    let mut stl_mesh = Node::new(NodeKind::Artifact, "stl_mesh", Origin::pass("output"))
        .with_attr("format", Value::Enum("stl".into()));
    stl_mesh.state = RefinementState::Realized;
    let stl_mesh = graph.add_node(stl_mesh);

    let mut gcode = Node::new(NodeKind::Artifact, "gcode", Origin::pass("output"))
        .with_attr("format", Value::Enum("gcode".into()));
    gcode.state = RefinementState::Realized;
    let gcode = graph.add_node(gcode);

    graph.add_edge(Edge::new(
        EdgeKind::Constrains,
        root,
        design_target,
        Origin::InputSpec,
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Contains,
        root,
        body,
        Origin::pass("synthesis"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::TargetsProcess,
        root,
        process,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::UsesMaterial,
        body,
        material,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::UsesMaterial,
        process,
        material,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Constrains,
        process,
        layer_height,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Constrains,
        process,
        tolerance,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Constrains,
        process,
        build_volume,
        Origin::pass("manufacturability"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Satisfies,
        body,
        design_target,
        Origin::pass("synthesis"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Produces,
        body,
        cad_model,
        Origin::pass("output"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Produces,
        body,
        stl_mesh,
        Origin::pass("output"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::Produces,
        body,
        gcode,
        Origin::pass("output"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::DerivedFrom,
        stl_mesh,
        cad_model,
        Origin::pass("output"),
    ))?;
    graph.add_edge(Edge::new(
        EdgeKind::DerivedFrom,
        gcode,
        stl_mesh,
        Origin::pass("output"),
    ))?;

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_id_by_name(graph: &IrGraph, name: &str) -> NodeId {
        *graph
            .nodes()
            .iter()
            .find(|(_, node)| node.name == name)
            .map(|(id, _)| id)
            .expect("node exists")
    }

    fn has_edge(graph: &IrGraph, kind: EdgeKind, source: NodeId, target: NodeId) -> bool {
        graph
            .edges()
            .values()
            .any(|edge| edge.kind == kind && edge.source == source && edge.target == target)
    }

    #[test]
    fn creates_root_node_with_expected_defaults() {
        let graph = IrGraph::new(NodeKind::System, "fidget_spinner", Origin::InputSpec);
        let root = graph.node(graph.root()).expect("root node exists");

        assert_eq!(graph.version(), IR_SCHEMA_VERSION);
        assert_eq!(root.kind, NodeKind::System);
        assert_eq!(root.state, RefinementState::Abstract);
        assert_eq!(root.origin, Origin::InputSpec);
    }

    #[test]
    fn stores_typed_attribute_values_with_units() {
        let node = Node::new(NodeKind::Constraint, "diameter", Origin::InputSpec)
            .with_attr(
                "target",
                Value::Quantity(Quantity {
                    value: 60.0,
                    unit: "mm".into(),
                }),
            )
            .with_attr(
                "tolerance",
                Value::Range(Range {
                    min: -0.2,
                    max: 0.2,
                    unit: "mm".into(),
                }),
            );

        assert_eq!(
            node.attrs["target"],
            Value::Quantity(Quantity {
                value: 60.0,
                unit: "mm".into()
            })
        );
        assert_eq!(
            node.attrs["tolerance"],
            Value::Range(Range {
                min: -0.2,
                max: 0.2,
                unit: "mm".into()
            })
        );
    }

    #[test]
    fn supports_typed_relationships_for_minimal_printable_prototype() {
        let mut graph = IrGraph::new(NodeKind::System, "fidget_spinner", Origin::InputSpec);
        let root = graph.root();

        let body = graph.add_node(Node::new(NodeKind::Part, "body", Origin::pass("synthesis")));
        let material = graph.add_node(Node::new(
            NodeKind::Material,
            "PLA",
            Origin::pass("manufacturability"),
        ));
        let process = graph.add_node(Node::new(
            NodeKind::Process,
            "fdm_printing",
            Origin::pass("manufacturability"),
        ));
        let artifact = graph.add_node(Node::new(
            NodeKind::Artifact,
            "stl_mesh",
            Origin::pass("output"),
        ));

        let contains = graph
            .add_edge(Edge::new(
                EdgeKind::Contains,
                root,
                body,
                Origin::pass("synthesis"),
            ))
            .expect("contains edge is valid");
        let uses_material = graph
            .add_edge(Edge::new(
                EdgeKind::UsesMaterial,
                body,
                material,
                Origin::pass("manufacturability"),
            ))
            .expect("uses_material edge is valid");
        let produces = graph
            .add_edge(Edge::new(
                EdgeKind::Produces,
                body,
                artifact,
                Origin::pass("output"),
            ))
            .expect("produces edge is valid");
        graph
            .add_edge(Edge::new(
                EdgeKind::TargetsProcess,
                root,
                process,
                Origin::pass("manufacturability"),
            ))
            .expect("targets_process edge is valid");

        assert_eq!(
            graph.edge(contains).expect("contains edge").kind,
            EdgeKind::Contains
        );
        assert_eq!(
            graph.edge(uses_material).expect("material edge").kind,
            EdgeKind::UsesMaterial
        );
        assert_eq!(
            graph.edge(produces).expect("artifact edge").kind,
            EdgeKind::Produces
        );
        assert_eq!(graph.nodes().len(), 5);
        assert_eq!(graph.edges().len(), 4);
    }

    #[test]
    fn maps_readme_first_prototype_into_ir_graph() {
        let graph = first_prototype_graph().expect("prototype graph builds");
        let root = graph.root();
        let design_target = node_id_by_name(&graph, "design_target");
        let body = node_id_by_name(&graph, "body");
        let material = node_id_by_name(&graph, "PLA");
        let process = node_id_by_name(&graph, "fdm_printing");
        let tolerance = node_id_by_name(&graph, "tolerance");
        let gcode = node_id_by_name(&graph, "gcode");

        assert_eq!(
            graph.metadata()["prototype"],
            Value::String("README first prototype".into())
        );
        assert!(has_edge(&graph, EdgeKind::Constrains, root, design_target));
        assert!(has_edge(&graph, EdgeKind::Contains, root, body));
        assert!(has_edge(&graph, EdgeKind::TargetsProcess, root, process));
        assert!(has_edge(&graph, EdgeKind::UsesMaterial, body, material));
        assert!(has_edge(&graph, EdgeKind::UsesMaterial, process, material));
        assert!(has_edge(&graph, EdgeKind::Constrains, process, tolerance));
        assert!(has_edge(&graph, EdgeKind::Satisfies, body, design_target));
        assert!(has_edge(&graph, EdgeKind::Produces, body, gcode));
        assert_eq!(graph.nodes().len(), 11);
        assert_eq!(graph.edges().len(), 14);
    }

    #[test]
    fn keeps_the_first_prototype_within_simple_printable_scope() {
        let graph = first_prototype_graph().expect("prototype graph builds");
        let allowed_kinds = [
            NodeKind::System,
            NodeKind::Part,
            NodeKind::Constraint,
            NodeKind::Material,
            NodeKind::Process,
            NodeKind::Artifact,
        ];

        assert!(graph
            .nodes()
            .values()
            .all(|node| allowed_kinds.contains(&node.kind)));
        assert_eq!(
            graph.node(node_id_by_name(&graph, "design_target")),
            Some(
                &Node::new(NodeKind::Constraint, "design_target", Origin::InputSpec)
                    .with_attr("object", Value::String("fidget_spinner".into()))
                    .with_attr("category", Value::Enum("fidget_toy".into()))
                    .with_attr(
                        "diameter",
                        Value::Quantity(Quantity {
                            value: 60.0,
                            unit: "mm".into(),
                        })
                    )
                    .with_attr("electronics_required", Value::Bool(false))
            )
        );
    }
}
