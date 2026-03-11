//! Manufacturing environment abstractions for the Matter compiler.
//! This crate is intentionally limited to manufacturing context concerns.

pub mod schema {
    /// Root type describing a concrete manufacturing target and its constraints.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ManufacturingContext {
        pub target: ManufacturingTarget,
        pub constraints: ManufacturingConstraints,
    }

    impl ManufacturingContext {
        /// Creates a manufacturing context from its target and constraints parts.
        ///
        /// ```rust
        /// use matter_context::{
        ///     AvailableTool, BuildVolume, LayerHeight, MachineIdentity, ManufacturingConstraints,
        ///     ManufacturingContext, ManufacturingEnvironment, ManufacturingTarget, Material,
        ///     ProcessType, Tolerance,
        /// };
        ///
        /// let target = ManufacturingTarget::new(
        ///     ManufacturingEnvironment::new("Home workshop"),
        ///     ProcessType::FusedDepositionModeling,
        ///     MachineIdentity::new("Bambu Lab", "P1S"),
        /// );
        ///
        /// let constraints = ManufacturingConstraints::new(
        ///     BuildVolume::new(256, 256, 256).unwrap(),
        ///     vec![Material::Pla],
        ///     LayerHeight::new_mm(0.2).unwrap(),
        ///     Tolerance::plus_minus_mm(0.2).unwrap(),
        ///     vec![AvailableTool::FusedDepositionPrinter],
        /// )
        /// .unwrap();
        ///
        /// let context = ManufacturingContext::new(target, constraints);
        ///
        /// assert_eq!(context, ManufacturingContext::initial_prototype());
        /// ```
        pub fn new(target: ManufacturingTarget, constraints: ManufacturingConstraints) -> Self {
            Self {
                target,
                constraints,
            }
        }

        /// Canonical manufacturing context for the repository's initial prototype.
        pub fn initial_prototype() -> Self {
            let target = ManufacturingTarget::new(
                ManufacturingEnvironment::new("Home workshop"),
                ProcessType::FusedDepositionModeling,
                MachineIdentity::new("Bambu Lab", "P1S"),
            );

            let constraints = ManufacturingConstraints::new(
                BuildVolume::new(256, 256, 256).expect("prototype build volume is valid"),
                vec![Material::Pla],
                LayerHeight::new_mm(0.2).expect("prototype layer height is valid"),
                Tolerance::plus_minus_mm(0.2).expect("prototype tolerance is valid"),
                vec![AvailableTool::FusedDepositionPrinter],
            )
            .expect("prototype constraints are valid");

            Self::new(target, constraints)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ManufacturingTarget {
        pub environment: ManufacturingEnvironment,
        pub process: ProcessType,
        pub machine: MachineIdentity,
    }

    impl ManufacturingTarget {
        pub fn new(
            environment: ManufacturingEnvironment,
            process: ProcessType,
            machine: MachineIdentity,
        ) -> Self {
            Self {
                environment,
                process,
                machine,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ManufacturingEnvironment {
        pub name: String,
    }

    impl ManufacturingEnvironment {
        pub fn new(name: impl Into<String>) -> Self {
            Self { name: name.into() }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MachineIdentity {
        pub manufacturer: String,
        pub model: String,
    }

    impl MachineIdentity {
        pub fn new(manufacturer: impl Into<String>, model: impl Into<String>) -> Self {
            Self {
                manufacturer: manufacturer.into(),
                model: model.into(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ManufacturingConstraints {
        pub build_volume: BuildVolume,
        pub supported_materials: Vec<Material>,
        pub nominal_layer_height: LayerHeight,
        pub achievable_tolerance: Tolerance,
        pub available_tools: Vec<AvailableTool>,
    }

    impl ManufacturingConstraints {
        pub fn new(
            build_volume: BuildVolume,
            supported_materials: Vec<Material>,
            nominal_layer_height: LayerHeight,
            achievable_tolerance: Tolerance,
            available_tools: Vec<AvailableTool>,
        ) -> Result<Self, SchemaError> {
            if supported_materials.is_empty() {
                return Err(SchemaError::MissingSupportedMaterials);
            }

            if available_tools.is_empty() {
                return Err(SchemaError::MissingAvailableTools);
            }

            Ok(Self {
                build_volume,
                supported_materials,
                nominal_layer_height,
                achievable_tolerance,
                available_tools,
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessType {
        FusedDepositionModeling,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Material {
        Pla,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AvailableTool {
        FusedDepositionPrinter,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BuildVolume {
        pub x_mm: u16,
        pub y_mm: u16,
        pub z_mm: u16,
    }

    impl BuildVolume {
        pub fn new(x_mm: u16, y_mm: u16, z_mm: u16) -> Result<Self, SchemaError> {
            if x_mm == 0 || y_mm == 0 || z_mm == 0 {
                return Err(SchemaError::BuildVolumeMustBePositive);
            }

            Ok(Self { x_mm, y_mm, z_mm })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct LayerHeight {
        pub millimeters: f32,
    }

    impl LayerHeight {
        pub fn new_mm(millimeters: f32) -> Result<Self, SchemaError> {
            if millimeters <= 0.0 {
                return Err(SchemaError::LayerHeightMustBePositive);
            }

            Ok(Self { millimeters })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Tolerance {
        pub plus_minus_mm: f32,
    }

    impl Tolerance {
        pub fn plus_minus_mm(plus_minus_mm: f32) -> Result<Self, SchemaError> {
            if plus_minus_mm <= 0.0 {
                return Err(SchemaError::ToleranceMustBePositive);
            }

            Ok(Self { plus_minus_mm })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SchemaError {
        BuildVolumeMustBePositive,
        LayerHeightMustBePositive,
        ToleranceMustBePositive,
        MissingSupportedMaterials,
        MissingAvailableTools,
    }
}

pub use schema::{
    AvailableTool, BuildVolume, LayerHeight, MachineIdentity, ManufacturingConstraints,
    ManufacturingContext, ManufacturingEnvironment, ManufacturingTarget, Material, ProcessType,
    SchemaError, Tolerance,
};

#[cfg(test)]
mod tests {
    use super::{
        AvailableTool, BuildVolume, LayerHeight, ManufacturingConstraints, ManufacturingContext,
        Material, ProcessType, SchemaError, Tolerance,
    };

    #[test]
    fn exposes_the_initial_prototype_context() {
        let prototype = ManufacturingContext::initial_prototype();

        assert_eq!(prototype.target.environment.name, "Home workshop");
        assert_eq!(
            prototype.target.process,
            ProcessType::FusedDepositionModeling
        );
        assert_eq!(prototype.target.machine.manufacturer, "Bambu Lab");
        assert_eq!(prototype.target.machine.model, "P1S");
        assert_eq!(prototype.constraints.build_volume.x_mm, 256);
        assert_eq!(prototype.constraints.build_volume.y_mm, 256);
        assert_eq!(prototype.constraints.build_volume.z_mm, 256);
        assert_eq!(
            prototype.constraints.supported_materials,
            vec![Material::Pla]
        );
        assert_eq!(prototype.constraints.nominal_layer_height.millimeters, 0.2);
        assert_eq!(
            prototype.constraints.achievable_tolerance.plus_minus_mm,
            0.2
        );
        assert_eq!(
            prototype.constraints.available_tools,
            vec![AvailableTool::FusedDepositionPrinter]
        );
    }

    #[test]
    fn rejects_invalid_constraint_inputs() {
        assert_eq!(
            BuildVolume::new(0, 256, 256),
            Err(SchemaError::BuildVolumeMustBePositive)
        );
        assert_eq!(
            LayerHeight::new_mm(0.0),
            Err(SchemaError::LayerHeightMustBePositive)
        );
        assert_eq!(
            Tolerance::plus_minus_mm(0.0),
            Err(SchemaError::ToleranceMustBePositive)
        );
        assert_eq!(
            ManufacturingConstraints::new(
                BuildVolume::new(256, 256, 256).unwrap(),
                Vec::new(),
                LayerHeight::new_mm(0.2).unwrap(),
                Tolerance::plus_minus_mm(0.2).unwrap(),
                vec![AvailableTool::FusedDepositionPrinter],
            ),
            Err(SchemaError::MissingSupportedMaterials)
        );
        assert_eq!(
            ManufacturingConstraints::new(
                BuildVolume::new(256, 256, 256).unwrap(),
                vec![Material::Pla],
                LayerHeight::new_mm(0.2).unwrap(),
                Tolerance::plus_minus_mm(0.2).unwrap(),
                Vec::new(),
            ),
            Err(SchemaError::MissingAvailableTools)
        );
    }

    #[test]
    fn explicit_construction_matches_the_canonical_prototype() {
        let target = super::ManufacturingTarget::new(
            super::ManufacturingEnvironment::new("Home workshop"),
            ProcessType::FusedDepositionModeling,
            super::MachineIdentity::new("Bambu Lab", "P1S"),
        );

        let constraints = ManufacturingConstraints::new(
            BuildVolume::new(256, 256, 256).unwrap(),
            vec![Material::Pla],
            LayerHeight::new_mm(0.2).unwrap(),
            Tolerance::plus_minus_mm(0.2).unwrap(),
            vec![AvailableTool::FusedDepositionPrinter],
        )
        .unwrap();

        let constructed = ManufacturingContext::new(target, constraints);

        assert_eq!(constructed, ManufacturingContext::initial_prototype());
    }
}
