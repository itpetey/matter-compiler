use matter_context::{ManufacturingContext, Material};
use matter_sdk::{
    Assembly, AuthorKind, Connection, ConnectionKind, Constraint, Design, Interface, InterfaceKind,
    InterfaceRef, Part, Provenance, Requirement, TypedValue,
};

pub fn build_design() -> Design {
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
                    .with_interface(Interface::new("cap_spigot", InterfaceKind::Mechanical))
                    .with_constraint(Constraint::new(
                        "retention_depth",
                        TypedValue::LengthMm(4.0),
                    )),
            )
            .with_connection(Connection::new(
                "cap_join",
                InterfaceRef::new("body", "cap_socket"),
                InterfaceRef::new("cap", "cap_spigot"),
                ConnectionKind::Fixed,
            )),
        ManufacturingContext::initial_prototype(),
        Provenance::new(
            AuthorKind::Human,
            "prototype-example",
            "crates/design-sdk/examples/fidget_spinner.rs",
        ),
    )
    .with_requirement(Requirement::new(
        "artefact_class",
        TypedValue::Text("fidget_toy".into()),
    ))
    .with_requirement(Requirement::new("max_diameter", TypedValue::LengthMm(60.0)))
}

#[allow(dead_code)]
fn main() {
    let design = build_design();
    let lowered = design.lower_to_ir().expect("example design lowers");

    println!("Example: {}", design.name);
    println!(
        "Target: {} {} via {:?}",
        lowered.manufacturing.target.machine.manufacturer,
        lowered.manufacturing.target.machine.model,
        lowered.manufacturing.target.process,
    );
    println!(
        "Supported materials: {:?}",
        lowered.manufacturing.constraints.supported_materials
    );
    println!(
        "Lowered graph: {} nodes, {} edges",
        lowered.graph.nodes().len(),
        lowered.graph.edges().len()
    );
}
