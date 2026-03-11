//! Minimal author-facing domain model for describing simple mechanical artefacts.

use std::collections::{BTreeMap, BTreeSet};

use matter_context::{ManufacturingContext, Material};
use matter_ir::{
    Edge, EdgeKind, GraphError, IrGraph, Node, NodeId, NodeKind, Origin, Quantity, RefinementState,
    Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Design {
    pub name: String,
    pub root: Assembly,
    pub requirements: Vec<Requirement>,
    pub manufacturing: ManufacturingContext,
    pub provenance: Provenance,
}

impl Design {
    pub fn new(
        name: impl Into<String>,
        root: Assembly,
        manufacturing: ManufacturingContext,
        provenance: Provenance,
    ) -> Self {
        Self {
            name: name.into(),
            root,
            requirements: Vec::new(),
            manufacturing,
            provenance,
        }
    }

    pub fn with_requirement(mut self, requirement: Requirement) -> Self {
        self.requirements.push(requirement);
        self
    }

    /// Lowers the validated SDK model into canonical compiler inputs.
    pub fn lower_to_ir(&self) -> Result<LoweredDesign, LoweringError> {
        self.validate()?;
        GraphLowerer::new(self).lower()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_label(&self.name, "design")?;
        self.provenance.validate()?;
        for requirement in &self.requirements {
            requirement.validate()?;
        }

        let mut interfaces_by_component = BTreeMap::new();
        let mut part_materials = Vec::new();
        self.root
            .collect_components(&mut interfaces_by_component, &mut part_materials)?;
        self.root.validate_connections(&interfaces_by_component)?;

        for (part, material) in part_materials {
            if !self
                .manufacturing
                .constraints
                .supported_materials
                .contains(&material)
            {
                return Err(ValidationError::UnsupportedMaterial { part, material });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assembly {
    pub id: String,
    pub components: Vec<Component>,
    pub interfaces: Vec<Interface>,
    pub connections: Vec<Connection>,
    pub constraints: Vec<Constraint>,
}

impl Assembly {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            components: Vec::new(),
            interfaces: Vec::new(),
            connections: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn with_component(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }

    pub fn with_part(self, part: Part) -> Self {
        self.with_component(Component::Part(part))
    }

    pub fn with_assembly(self, assembly: Assembly) -> Self {
        self.with_component(Component::Assembly(assembly))
    }

    pub fn with_interface(mut self, interface: Interface) -> Self {
        self.interfaces.push(interface);
        self
    }

    pub fn with_connection(mut self, connection: Connection) -> Self {
        self.connections.push(connection);
        self
    }

    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    fn collect_components(
        &self,
        interfaces_by_component: &mut BTreeMap<String, BTreeSet<String>>,
        part_materials: &mut Vec<(String, Material)>,
    ) -> Result<(), ValidationError> {
        validate_label(&self.id, "assembly")?;
        if self.components.is_empty() {
            return Err(ValidationError::EmptyAssembly(self.id.clone()));
        }

        for constraint in &self.constraints {
            constraint.validate()?;
        }

        register_component(&self.id, &self.interfaces, interfaces_by_component)?;

        for component in &self.components {
            match component {
                Component::Assembly(assembly) => {
                    assembly.collect_components(interfaces_by_component, part_materials)?;
                }
                Component::Part(part) => {
                    part.collect_components(interfaces_by_component, part_materials)?;
                }
            }
        }

        Ok(())
    }

    fn validate_connections(
        &self,
        interfaces_by_component: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<(), ValidationError> {
        for connection in &self.connections {
            connection.validate(interfaces_by_component)?;
        }

        for component in &self.components {
            if let Component::Assembly(assembly) = component {
                assembly.validate_connections(interfaces_by_component)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    Assembly(Assembly),
    Part(Part),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub id: String,
    pub material: Material,
    pub interfaces: Vec<Interface>,
    pub constraints: Vec<Constraint>,
}

impl Part {
    pub fn new(id: impl Into<String>, material: Material) -> Self {
        Self {
            id: id.into(),
            material,
            interfaces: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn with_interface(mut self, interface: Interface) -> Self {
        self.interfaces.push(interface);
        self
    }

    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    fn collect_components(
        &self,
        interfaces_by_component: &mut BTreeMap<String, BTreeSet<String>>,
        part_materials: &mut Vec<(String, Material)>,
    ) -> Result<(), ValidationError> {
        validate_label(&self.id, "part")?;
        for constraint in &self.constraints {
            constraint.validate()?;
        }

        register_component(&self.id, &self.interfaces, interfaces_by_component)?;
        part_materials.push((self.id.clone(), self.material));
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub name: String,
    pub kind: InterfaceKind,
}

impl Interface {
    pub fn new(name: impl Into<String>, kind: InterfaceKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceKind {
    Mechanical,
    ElectricalPlaceholder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub name: String,
    pub from: InterfaceRef,
    pub to: InterfaceRef,
    pub kind: ConnectionKind,
}

impl Connection {
    pub fn new(
        name: impl Into<String>,
        from: InterfaceRef,
        to: InterfaceRef,
        kind: ConnectionKind,
    ) -> Self {
        Self {
            name: name.into(),
            from,
            to,
            kind,
        }
    }

    fn validate(
        &self,
        interfaces_by_component: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<(), ValidationError> {
        validate_label(&self.name, "connection")?;
        self.from.validate(interfaces_by_component, &self.name)?;
        self.to.validate(interfaces_by_component, &self.name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionKind {
    Fixed,
    Rotational,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRef {
    pub component: String,
    pub interface: String,
}

impl InterfaceRef {
    pub fn new(component: impl Into<String>, interface: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            interface: interface.into(),
        }
    }

    fn validate(
        &self,
        interfaces_by_component: &BTreeMap<String, BTreeSet<String>>,
        connection: &str,
    ) -> Result<(), ValidationError> {
        let interfaces = interfaces_by_component
            .get(&self.component)
            .ok_or_else(|| ValidationError::MissingConnectionComponent {
                connection: connection.into(),
                component: self.component.clone(),
            })?;

        if !interfaces.contains(&self.interface) {
            return Err(ValidationError::MissingConnectionInterface {
                connection: connection.into(),
                component: self.component.clone(),
                interface: self.interface.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Requirement {
    pub name: String,
    pub value: TypedValue,
}

impl Requirement {
    pub fn new(name: impl Into<String>, value: TypedValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    fn validate(&self) -> Result<(), ValidationError> {
        validate_named_value(&self.name, &self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub name: String,
    pub value: TypedValue,
}

impl Constraint {
    pub fn new(name: impl Into<String>, value: TypedValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    fn validate(&self) -> Result<(), ValidationError> {
        validate_named_value(&self.name, &self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Boolean(bool),
    Count(u32),
    LengthMm(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub author_kind: AuthorKind,
    pub author: String,
    pub source: String,
}

impl Provenance {
    pub fn new(
        author_kind: AuthorKind,
        author: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            author_kind,
            author: author.into(),
            source: source.into(),
        }
    }

    fn validate(&self) -> Result<(), ValidationError> {
        validate_label(&self.author, "provenance author")?;
        validate_label(&self.source, "provenance source")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorKind {
    Human,
    Ai,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoweredDesign {
    pub graph: IrGraph,
    pub manufacturing: ManufacturingContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    EmptyName(&'static str),
    EmptyAssembly(String),
    DuplicateComponentId(String),
    DuplicateInterfaceId {
        component: String,
        interface: String,
    },
    MissingConnectionComponent {
        connection: String,
        component: String,
    },
    MissingConnectionInterface {
        connection: String,
        component: String,
        interface: String,
    },
    UnsupportedMaterial {
        part: String,
        material: Material,
    },
    InvalidLength {
        name: String,
        value_mm: f64,
    },
}

fn register_component(
    component: &str,
    interfaces: &[Interface],
    interfaces_by_component: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), ValidationError> {
    let mut interface_names = BTreeSet::new();
    for interface in interfaces {
        validate_label(&interface.name, "interface")?;
        if !interface_names.insert(interface.name.clone()) {
            return Err(ValidationError::DuplicateInterfaceId {
                component: component.into(),
                interface: interface.name.clone(),
            });
        }
    }

    if interfaces_by_component
        .insert(component.into(), interface_names)
        .is_some()
    {
        return Err(ValidationError::DuplicateComponentId(component.into()));
    }

    Ok(())
}

fn validate_label(value: &str, entity: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyName(entity));
    }

    Ok(())
}

fn validate_named_value(name: &str, value: &TypedValue) -> Result<(), ValidationError> {
    validate_label(name, "named value")?;
    if let TypedValue::LengthMm(value_mm) = value {
        if *value_mm <= 0.0 {
            return Err(ValidationError::InvalidLength {
                name: name.into(),
                value_mm: *value_mm,
            });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoweringError {
    Validation(ValidationError),
    Graph(GraphError),
}

impl From<ValidationError> for LoweringError {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<GraphError> for LoweringError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

struct PendingConnection<'a> {
    path: String,
    connection: &'a Connection,
}

struct GraphLowerer<'a> {
    design: &'a Design,
    graph: IrGraph,
    interface_nodes: BTreeMap<(String, String), NodeId>,
    material_nodes: BTreeMap<String, NodeId>,
    pending_connections: Vec<PendingConnection<'a>>,
}

impl<'a> GraphLowerer<'a> {
    fn new(design: &'a Design) -> Self {
        let mut graph = IrGraph::new(NodeKind::System, design.name.clone(), Origin::Sdk);
        let metadata = graph.metadata_mut();
        metadata.insert(
            "sdk.lowering_contract".into(),
            Value::String("design-sdk:minimal-v1".into()),
        );
        metadata.insert(
            "sdk.author_kind".into(),
            Value::Enum(author_kind_label(&design.provenance.author_kind).into()),
        );
        metadata.insert(
            "sdk.author".into(),
            Value::String(design.provenance.author.clone()),
        );
        metadata.insert(
            "sdk.source".into(),
            Value::String(design.provenance.source.clone()),
        );

        Self {
            design,
            graph,
            interface_nodes: BTreeMap::new(),
            material_nodes: BTreeMap::new(),
            pending_connections: Vec::new(),
        }
    }

    fn lower(mut self) -> Result<LoweredDesign, LoweringError> {
        let root = self.graph.root();
        self.lower_assembly(root, &self.design.root, "design.root")?;

        for (index, requirement) in self.design.requirements.iter().enumerate() {
            self.lower_named_value(
                root,
                &requirement.name,
                &requirement.value,
                &format!("design.requirements[{index}]"),
            );
        }

        for pending in std::mem::take(&mut self.pending_connections) {
            let from = self.interface_node_id(&pending.connection.from)?;
            let to = self.interface_node_id(&pending.connection.to)?;
            self.graph.add_edge(
                Edge::new(EdgeKind::Connects, from, to, Origin::Sdk)
                    .with_attr("name", Value::String(pending.connection.name.clone()))
                    .with_attr(
                        "connection_kind",
                        Value::Enum(connection_kind_label(&pending.connection.kind).into()),
                    )
                    .with_attr("sdk.authoring_path", Value::String(pending.path)),
            )?;
        }

        Ok(LoweredDesign {
            graph: self.graph,
            manufacturing: self.design.manufacturing.clone(),
        })
    }

    fn lower_assembly(
        &mut self,
        parent: NodeId,
        assembly: &'a Assembly,
        path: &str,
    ) -> Result<NodeId, LoweringError> {
        let assembly_node = self.graph.add_node(
            Node::new(NodeKind::Assembly, assembly.id.clone(), Origin::Sdk)
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        );
        self.graph.add_edge(
            Edge::new(EdgeKind::Contains, parent, assembly_node, Origin::Sdk)
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        )?;

        for (index, interface) in assembly.interfaces.iter().enumerate() {
            self.lower_interface(
                &assembly.id,
                assembly_node,
                interface,
                &format!("{path}.interfaces[{index}]"),
            )?;
        }

        for (index, constraint) in assembly.constraints.iter().enumerate() {
            self.lower_named_value(
                assembly_node,
                &constraint.name,
                &constraint.value,
                &format!("{path}.constraints[{index}]"),
            );
        }

        for (index, component) in assembly.components.iter().enumerate() {
            let component_path = format!("{path}.components[{index}]");
            match component {
                Component::Assembly(child) => {
                    self.lower_assembly(assembly_node, child, &component_path)?;
                }
                Component::Part(part) => {
                    self.lower_part(assembly_node, part, &component_path)?;
                }
            }
        }

        for (index, connection) in assembly.connections.iter().enumerate() {
            self.pending_connections.push(PendingConnection {
                path: format!("{path}.connections[{index}]"),
                connection,
            });
        }

        Ok(assembly_node)
    }

    fn lower_part(
        &mut self,
        parent: NodeId,
        part: &'a Part,
        path: &str,
    ) -> Result<NodeId, LoweringError> {
        let part_node = self.graph.add_node(
            Node::new(NodeKind::Part, part.id.clone(), Origin::Sdk)
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        );
        self.graph.add_edge(
            Edge::new(EdgeKind::Contains, parent, part_node, Origin::Sdk)
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        )?;

        for (index, interface) in part.interfaces.iter().enumerate() {
            self.lower_interface(
                &part.id,
                part_node,
                interface,
                &format!("{path}.interfaces[{index}]"),
            )?;
        }

        for (index, constraint) in part.constraints.iter().enumerate() {
            self.lower_named_value(
                part_node,
                &constraint.name,
                &constraint.value,
                &format!("{path}.constraints[{index}]"),
            );
        }

        let material_node = self.material_node_id(part.material);
        self.graph.add_edge(
            Edge::new(
                EdgeKind::UsesMaterial,
                part_node,
                material_node,
                Origin::Sdk,
            )
            .with_attr(
                "sdk.authoring_path",
                Value::String(format!("{path}.material")),
            ),
        )?;

        Ok(part_node)
    }

    fn lower_interface(
        &mut self,
        component_id: &str,
        owner: NodeId,
        interface: &Interface,
        path: &str,
    ) -> Result<NodeId, LoweringError> {
        let interface_node = self.graph.add_node(
            Node::new(
                NodeKind::Interface,
                format!("{component_id}.{}", interface.name),
                Origin::Sdk,
            )
            .with_attr("component", Value::String(component_id.into()))
            .with_attr(
                "interface_kind",
                Value::Enum(interface_kind_label(&interface.kind).into()),
            )
            .with_attr("sdk.authoring_path", Value::String(path.into())),
        );
        self.graph.add_edge(
            Edge::new(EdgeKind::Contains, owner, interface_node, Origin::Sdk)
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        )?;
        self.interface_nodes.insert(
            (component_id.into(), interface.name.clone()),
            interface_node,
        );
        Ok(interface_node)
    }

    fn lower_named_value(&mut self, owner: NodeId, name: &str, value: &TypedValue, path: &str) {
        let constraint_node = self.graph.add_node(
            Node::new(NodeKind::Constraint, name.to_string(), Origin::Sdk)
                .with_attr("value", lower_typed_value(value))
                .with_attr("sdk.authoring_path", Value::String(path.into())),
        );
        self.graph
            .add_edge(
                Edge::new(EdgeKind::Constrains, owner, constraint_node, Origin::Sdk)
                    .with_attr("sdk.authoring_path", Value::String(path.into())),
            )
            .expect("lowering only references existing nodes");
    }

    fn interface_node_id(&self, interface: &InterfaceRef) -> Result<NodeId, LoweringError> {
        self.interface_nodes
            .get(&(interface.component.clone(), interface.interface.clone()))
            .copied()
            .ok_or_else(|| {
                LoweringError::Validation(ValidationError::MissingConnectionInterface {
                    connection: "lowering".into(),
                    component: interface.component.clone(),
                    interface: interface.interface.clone(),
                })
            })
    }

    fn material_node_id(&mut self, material: Material) -> NodeId {
        let material_name = material_label(material).to_string();
        if let Some(node_id) = self.material_nodes.get(&material_name) {
            return *node_id;
        }

        let mut material_node = Node::new(NodeKind::Material, material_name.clone(), Origin::Sdk);
        material_node.state = RefinementState::Selected;
        let node_id = self.graph.add_node(material_node);
        self.material_nodes.insert(material_name, node_id);
        node_id
    }
}

fn lower_typed_value(value: &TypedValue) -> Value {
    match value {
        TypedValue::Boolean(value) => Value::Bool(*value),
        TypedValue::Count(value) => Value::Int(i64::from(*value)),
        TypedValue::LengthMm(value) => Value::Quantity(Quantity {
            value: *value,
            unit: "mm".into(),
        }),
        TypedValue::Text(value) => Value::String(value.clone()),
    }
}

fn author_kind_label(author_kind: &AuthorKind) -> &'static str {
    match author_kind {
        AuthorKind::Human => "human",
        AuthorKind::Ai => "ai",
    }
}

fn connection_kind_label(connection_kind: &ConnectionKind) -> &'static str {
    match connection_kind {
        ConnectionKind::Fixed => "fixed",
        ConnectionKind::Rotational => "rotational",
    }
}

fn interface_kind_label(interface_kind: &InterfaceKind) -> &'static str {
    match interface_kind {
        InterfaceKind::Mechanical => "mechanical",
        InterfaceKind::ElectricalPlaceholder => "electrical_placeholder",
    }
}

fn material_label(material: Material) -> &'static str {
    match material {
        Material::Pla => "PLA",
    }
}

#[cfg(test)]
mod tests {
    use matter_context::{
        AvailableTool, BuildVolume, LayerHeight, MachineIdentity, ManufacturingConstraints,
        ManufacturingEnvironment, ManufacturingTarget, ProcessType, Tolerance,
    };
    use matter_ir::{EdgeKind, IrGraph, NodeId, NodeKind, Origin, RefinementState, Value};

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
    fn validates_a_minimal_mechanical_design_without_exposing_ir() {
        let design = Design::new(
            "fidget_spinner",
            Assembly::new("spinner")
                .with_component(Component::Part(
                    Part::new("body", Material::Pla)
                        .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical))
                        .with_constraint(Constraint::new("diameter", TypedValue::LengthMm(60.0))),
                ))
                .with_component(Component::Part(
                    Part::new("cap", Material::Pla)
                        .with_interface(Interface::new("cap_spigot", InterfaceKind::Mechanical)),
                ))
                .with_connection(Connection::new(
                    "cap_join",
                    InterfaceRef::new("body", "cap_socket"),
                    InterfaceRef::new("cap", "cap_spigot"),
                    ConnectionKind::Fixed,
                )),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "README first prototype"),
        )
        .with_requirement(Requirement::new(
            "artefact_class",
            TypedValue::Text("fidget_toy".into()),
        ))
        .with_requirement(Requirement::new("diameter", TypedValue::LengthMm(60.0)));

        assert_eq!(design.validate(), Ok(()));
    }

    #[test]
    fn rejects_connections_to_interfaces_that_do_not_exist() {
        let design = Design::new(
            "invalid_join",
            Assembly::new("spinner")
                .with_component(Component::Part(
                    Part::new("body", Material::Pla)
                        .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical)),
                ))
                .with_component(Component::Part(
                    Part::new("cap", Material::Pla)
                        .with_interface(Interface::new("cap_spigot", InterfaceKind::Mechanical)),
                ))
                .with_connection(Connection::new(
                    "cap_join",
                    InterfaceRef::new("body", "missing_socket"),
                    InterfaceRef::new("cap", "cap_spigot"),
                    ConnectionKind::Fixed,
                )),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Ai, "design-agent", "task prompt"),
        );

        assert_eq!(
            design.validate(),
            Err(ValidationError::MissingConnectionInterface {
                connection: "cap_join".into(),
                component: "body".into(),
                interface: "missing_socket".into(),
            })
        );
    }

    #[test]
    fn rejects_part_materials_that_are_not_supported_by_the_target_context() {
        let unsupported_context = ManufacturingContext {
            target: ManufacturingTarget::new(
                ManufacturingEnvironment::new("Home workshop"),
                ProcessType::FusedDepositionModeling,
                MachineIdentity::new("Bambu Lab", "P1S"),
            ),
            constraints: ManufacturingConstraints::new(
                BuildVolume::new(256, 256, 256).unwrap(),
                vec![Material::Pla],
                LayerHeight::new_mm(0.2).unwrap(),
                Tolerance::plus_minus_mm(0.2).unwrap(),
                vec![AvailableTool::FusedDepositionPrinter],
            )
            .unwrap(),
        };

        let mut unsupported_context = unsupported_context;
        unsupported_context.constraints.supported_materials.clear();

        let design = Design::new(
            "unsupported_material",
            Assembly::new("spinner")
                .with_component(Component::Part(Part::new("body", Material::Pla))),
            unsupported_context,
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(
            design.validate(),
            Err(ValidationError::UnsupportedMaterial {
                part: "body".into(),
                material: Material::Pla,
            })
        );
    }

    #[test]
    fn rejects_blank_design_names() {
        let design = Design::new(
            "   ",
            Assembly::new("spinner")
                .with_component(Component::Part(Part::new("body", Material::Pla))),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(design.validate(), Err(ValidationError::EmptyName("design")));
    }

    #[test]
    fn rejects_empty_root_assemblies() {
        let design = Design::new(
            "empty_spinner",
            Assembly::new("spinner"),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(
            design.validate(),
            Err(ValidationError::EmptyAssembly("spinner".into()))
        );
    }

    #[test]
    fn rejects_duplicate_component_ids() {
        let design = Design::new(
            "duplicate_parts",
            Assembly::new("spinner")
                .with_component(Component::Part(Part::new("body", Material::Pla)))
                .with_component(Component::Part(Part::new("body", Material::Pla))),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(
            design.validate(),
            Err(ValidationError::DuplicateComponentId("body".into()))
        );
    }

    #[test]
    fn rejects_duplicate_interface_ids_within_a_component() {
        let design = Design::new(
            "duplicate_interfaces",
            Assembly::new("spinner").with_component(Component::Part(
                Part::new("body", Material::Pla)
                    .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical))
                    .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical)),
            )),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(
            design.validate(),
            Err(ValidationError::DuplicateInterfaceId {
                component: "body".into(),
                interface: "cap_socket".into(),
            })
        );
    }

    #[test]
    fn rejects_non_positive_length_requirements() {
        let design = Design::new(
            "invalid_requirement",
            Assembly::new("spinner")
                .with_component(Component::Part(Part::new("body", Material::Pla))),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        )
        .with_requirement(Requirement::new("diameter", TypedValue::LengthMm(0.0)));

        assert_eq!(
            design.validate(),
            Err(ValidationError::InvalidLength {
                name: "diameter".into(),
                value_mm: 0.0,
            })
        );
    }

    #[test]
    fn lowers_a_minimal_authored_design_into_deterministic_ir() {
        let design = Design::new(
            "fidget_spinner",
            Assembly::new("spinner")
                .with_part(
                    Part::new("body", Material::Pla)
                        .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical))
                        .with_constraint(Constraint::new(
                            "outer_diameter",
                            TypedValue::LengthMm(60.0),
                        )),
                )
                .with_part(
                    Part::new("cap", Material::Pla)
                        .with_interface(Interface::new("cap_spigot", InterfaceKind::Mechanical)),
                )
                .with_connection(Connection::new(
                    "cap_join",
                    InterfaceRef::new("body", "cap_socket"),
                    InterfaceRef::new("cap", "cap_spigot"),
                    ConnectionKind::Fixed,
                )),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Ai, "design-agent", "task prompt"),
        )
        .with_requirement(Requirement::new(
            "artefact_class",
            TypedValue::Text("fidget_toy".into()),
        ));

        let first = design.lower_to_ir().expect("lowering succeeds");
        let second = design.lower_to_ir().expect("lowering is deterministic");
        let graph = &first.graph;
        let root = graph.root();
        let spinner = node_id_by_name(graph, "spinner");
        let body = node_id_by_name(graph, "body");
        let cap = node_id_by_name(graph, "cap");
        let body_socket = node_id_by_name(graph, "body.cap_socket");
        let cap_spigot = node_id_by_name(graph, "cap.cap_spigot");
        let outer_diameter = node_id_by_name(graph, "outer_diameter");
        let artefact_class = node_id_by_name(graph, "artefact_class");
        let pla = node_id_by_name(graph, "PLA");

        assert_eq!(first, second);
        assert_eq!(
            first.manufacturing,
            ManufacturingContext::initial_prototype()
        );
        assert_eq!(
            graph.metadata()["sdk.lowering_contract"],
            Value::String("design-sdk:minimal-v1".into())
        );
        assert_eq!(
            graph.metadata()["sdk.author_kind"],
            Value::Enum("ai".into())
        );
        assert_eq!(
            graph.metadata()["sdk.author"],
            Value::String("design-agent".into())
        );
        assert_eq!(
            graph.metadata()["sdk.source"],
            Value::String("task prompt".into())
        );
        assert_eq!(graph.node(root).expect("root node").kind, NodeKind::System);
        assert_eq!(graph.node(root).expect("root node").origin, Origin::Sdk);
        assert!(has_edge(graph, EdgeKind::Contains, root, spinner));
        assert!(has_edge(graph, EdgeKind::Contains, spinner, body));
        assert!(has_edge(graph, EdgeKind::Contains, spinner, cap));
        assert!(has_edge(graph, EdgeKind::Contains, body, body_socket));
        assert!(has_edge(graph, EdgeKind::Contains, cap, cap_spigot));
        assert!(has_edge(graph, EdgeKind::Constrains, body, outer_diameter));
        assert!(has_edge(graph, EdgeKind::Constrains, root, artefact_class));
        assert!(has_edge(graph, EdgeKind::UsesMaterial, body, pla));
        assert!(has_edge(graph, EdgeKind::UsesMaterial, cap, pla));
        assert!(has_edge(graph, EdgeKind::Connects, body_socket, cap_spigot));
        assert_eq!(
            graph.node(pla).expect("material node").state,
            RefinementState::Selected
        );
        assert_eq!(
            graph.node(body).expect("body node").attrs["sdk.authoring_path"],
            Value::String("design.root.components[0]".into())
        );
        assert_eq!(
            graph.node(body_socket).expect("body interface").attrs["sdk.authoring_path"],
            Value::String("design.root.components[0].interfaces[0]".into())
        );
        assert_eq!(
            graph.node(outer_diameter).expect("body constraint").attrs["value"],
            Value::Quantity(Quantity {
                value: 60.0,
                unit: "mm".into(),
            })
        );
        let connects = graph
            .edges()
            .values()
            .find(|edge| edge.kind == EdgeKind::Connects)
            .expect("connects edge exists");
        assert_eq!(connects.attrs["name"], Value::String("cap_join".into()));
        assert_eq!(
            connects.attrs["sdk.authoring_path"],
            Value::String("design.root.connections[0]".into())
        );
        assert_eq!(graph.nodes().len(), 9);
        assert_eq!(graph.edges().len(), 10);
    }

    #[test]
    fn lowering_reports_validation_failures_without_partial_ir() {
        let design = Design::new(
            "invalid_join",
            Assembly::new("spinner")
                .with_part(
                    Part::new("body", Material::Pla)
                        .with_interface(Interface::new("cap_socket", InterfaceKind::Mechanical)),
                )
                .with_part(
                    Part::new("cap", Material::Pla)
                        .with_interface(Interface::new("cap_spigot", InterfaceKind::Mechanical)),
                )
                .with_connection(Connection::new(
                    "cap_join",
                    InterfaceRef::new("body", "missing_socket"),
                    InterfaceRef::new("cap", "cap_spigot"),
                    ConnectionKind::Fixed,
                )),
            ManufacturingContext::initial_prototype(),
            Provenance::new(AuthorKind::Human, "Pete", "test"),
        );

        assert_eq!(
            design.lower_to_ir(),
            Err(LoweringError::Validation(
                ValidationError::MissingConnectionInterface {
                    connection: "cap_join".into(),
                    component: "body".into(),
                    interface: "missing_socket".into(),
                }
            ))
        );
    }
}
