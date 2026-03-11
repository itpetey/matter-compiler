use matter_compiler::compile_prototype;
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
            "crates/sdk/examples/fidget_spinner.rs",
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
    let compiled = compile_prototype(&design).expect("example design compiles");

    println!("Example: {}", design.name);
    println!(
        "Target: {} {} via {:?}",
        compiled.plan.machine.manufacturer, compiled.plan.machine.model, compiled.plan.process,
    );
    println!("Material: {:?}", compiled.plan.material);
    println!("Validated parts: {}", compiled.report.validated_part_count);
    println!("Executed passes: {:?}", compiled.report.executed_passes);
    println!(
        "Compiled graph: {} nodes, {} edges",
        compiled.node_count(),
        compiled.edge_count()
    );
}
