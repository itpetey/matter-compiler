//! Standalone fidget spinner example built with the Matter SDK.

use matter_context::{ManufacturingContext, Material};
use matter_sdk::{
    Assembly, AuthorKind, Connection, ConnectionKind, Constraint, Design, Interface, InterfaceKind,
    InterfaceRef, Part, Provenance, Requirement, TypedValue,
};

/// Builds the sample fidget spinner design used across the workspace.
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
            "examples/fidget_spinner/src/lib.rs",
        ),
    )
    .with_requirement(Requirement::new(
        "artefact_class",
        TypedValue::Text("fidget_toy".into()),
    ))
    .with_requirement(Requirement::new("max_diameter", TypedValue::LengthMm(60.0)))
}
