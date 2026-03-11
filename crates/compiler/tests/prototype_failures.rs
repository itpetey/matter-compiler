use matter_compiler::{CompileError, compile_lowered_prototype, compile_prototype};
use matter_context::{MachineIdentity, ManufacturingContext, Material};
use matter_ir::{Edge, EdgeKind, Node, NodeKind, Origin};
use matter_sdk::{
    Assembly, AuthorKind, Component, Connection, ConnectionKind, Constraint, Design, Interface,
    InterfaceKind, InterfaceRef, LoweringError, Part, Provenance, Requirement, TypedValue,
    ValidationError,
};

fn prototype_design() -> Design {
    Design::new(
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
        Provenance::new(AuthorKind::Human, "prototype-example", "test"),
    )
}

#[test]
fn rejects_non_canonical_machine_models() {
    let mut design = prototype_design();
    design.manufacturing.target.machine = MachineIdentity::new("Bambu Lab", "X1C");

    assert_eq!(
        compile_prototype(&design),
        Err(CompileError::UnsupportedPrototypeContext {
            field: "target.machine.model",
            expected: "\"P1S\"".into(),
            found: "\"X1C\"".into(),
        })
    );
}

#[test]
fn rejects_out_of_scope_interface_and_connection_kinds() {
    let mut electrical = prototype_design();
    electrical.root.components[0] = Component::Part(
        Part::new("body", Material::Pla)
            .with_interface(Interface::new(
                "cap_socket",
                InterfaceKind::ElectricalPlaceholder,
            ))
            .with_constraint(Constraint::new(
                "outer_diameter",
                TypedValue::LengthMm(60.0),
            )),
    );

    assert_eq!(
        compile_prototype(&electrical),
        Err(CompileError::UnsupportedInterfaceKind {
            interface: "body.cap_socket".into(),
            kind: "electrical_placeholder".into(),
        })
    );

    let rotational = Design::new(
        "spinner_joint",
        Assembly::new("spinner")
            .with_part(
                Part::new("body", Material::Pla)
                    .with_interface(Interface::new("hub", InterfaceKind::Mechanical)),
            )
            .with_part(
                Part::new("cap", Material::Pla)
                    .with_interface(Interface::new("bearing", InterfaceKind::Mechanical)),
            )
            .with_connection(Connection::new(
                "bearing_join",
                InterfaceRef::new("body", "hub"),
                InterfaceRef::new("cap", "bearing"),
                ConnectionKind::Rotational,
            )),
        ManufacturingContext::initial_prototype(),
        Provenance::new(AuthorKind::Human, "prototype-example", "test"),
    );

    assert_eq!(
        compile_prototype(&rotational),
        Err(CompileError::UnsupportedConnectionKind {
            connection: "bearing_join".into(),
            kind: "rotational".into(),
        })
    );
}

#[test]
fn rejects_dimensions_that_exceed_the_build_volume() {
    let design = prototype_design().with_requirement(Requirement::new(
        "oversized_diameter",
        TypedValue::LengthMm(300.0),
    ));

    assert_eq!(
        compile_prototype(&design),
        Err(CompileError::ExceedsBuildVolume {
            name: "oversized_diameter".into(),
            value_mm: 300.0,
            limit_mm: 256,
        })
    );
}

#[test]
fn rejects_later_stage_ir_shapes_and_propagates_lowering_failures() {
    let mut lowered = prototype_design().lower_to_ir().expect("design lowers");
    let process = lowered.graph.add_node(Node::new(
        NodeKind::Process,
        "fdm_printing",
        Origin::pass("manufacturability"),
    ));
    lowered
        .graph
        .add_edge(Edge::new(
            EdgeKind::TargetsProcess,
            lowered.graph.root(),
            process,
            Origin::pass("manufacturability"),
        ))
        .expect("edge is valid");

    assert_eq!(
        compile_lowered_prototype(lowered),
        Err(CompileError::UnsupportedNodeKind {
            kind: NodeKind::Process,
        })
    );

    let invalid = Design::new(
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
        Provenance::new(AuthorKind::Human, "prototype-example", "test"),
    );

    assert_eq!(
        compile_prototype(&invalid),
        Err(CompileError::Lowering(LoweringError::Validation(
            ValidationError::MissingConnectionInterface {
                connection: "cap_join".into(),
                component: "body".into(),
                interface: "missing_socket".into(),
            }
        )))
    );
}
