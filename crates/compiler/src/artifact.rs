use std::fmt::Write as _;

/// First-class printable artefact produced by the prototype compiler.
///
/// The current boundary is intentionally narrow: a stable artefact name plus an in-memory
/// triangle mesh that can be serialized to STL for the supported PLA-on-P1S spinner flow.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeArtifact {
    /// Stable identifier for the printable artefact.
    pub name: String,
    /// Export format supported by downstream serialization.
    pub format: PrototypeArtifactFormat,
    /// Deterministic mesh payload for the artefact.
    pub mesh: PrototypeTriangleMesh,
}

impl PrototypeArtifact {
    /// Creates an STL-oriented printable artefact from a deterministic triangle mesh.
    pub fn stl(name: impl Into<String>, mesh: PrototypeTriangleMesh) -> Self {
        Self {
            name: name.into(),
            format: PrototypeArtifactFormat::Stl,
            mesh,
        }
    }

    /// Serializes this artefact as deterministic ASCII STL content.
    pub fn to_ascii_stl(&self) -> String {
        self.mesh.to_ascii_stl(&self.name)
    }
}

/// Export formats supported by the prototype printable artefact boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeArtifactFormat {
    /// A triangle mesh intended for STL serialization.
    Stl,
}

/// Deterministic triangle mesh used by the prototype printable artefact boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeTriangleMesh {
    /// Length unit shared by every vertex in the mesh.
    pub units: PrototypeLengthUnit,
    /// Ordered triangle soup suitable for deterministic STL facet emission.
    pub triangles: Vec<PrototypeTriangle>,
}

impl PrototypeTriangleMesh {
    /// Creates a triangle mesh expressed in millimetres.
    pub fn new(triangles: Vec<PrototypeTriangle>) -> Self {
        Self {
            units: PrototypeLengthUnit::Millimetres,
            triangles,
        }
    }

    /// Serializes the mesh as deterministic ASCII STL content for the provided solid name.
    pub fn to_ascii_stl(&self, solid_name: &str) -> String {
        let mut output = String::new();
        writeln!(&mut output, "solid {solid_name}").expect("writing to a string cannot fail");

        for triangle in &self.triangles {
            let normal = triangle.normal();
            writeln!(
                &mut output,
                "  facet normal {} {} {}",
                format_stl_scalar(normal.x),
                format_stl_scalar(normal.y),
                format_stl_scalar(normal.z),
            )
            .expect("writing to a string cannot fail");
            writeln!(&mut output, "    outer loop").expect("writing to a string cannot fail");

            for vertex in triangle.vertices {
                writeln!(
                    &mut output,
                    "      vertex {} {} {}",
                    format_stl_scalar(vertex.x),
                    format_stl_scalar(vertex.y),
                    format_stl_scalar(vertex.z),
                )
                .expect("writing to a string cannot fail");
            }

            writeln!(&mut output, "    endloop").expect("writing to a string cannot fail");
            writeln!(&mut output, "  endfacet").expect("writing to a string cannot fail");
        }

        writeln!(&mut output, "endsolid {solid_name}").expect("writing to a string cannot fail");
        output
    }
}

/// Length units supported by the prototype printable artefact boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeLengthUnit {
    /// Millimetres, matching the current prototype manufacturing context and STL path.
    Millimetres,
}

/// Single triangle in the prototype mesh payload.
#[derive(Debug, Clone, PartialEq)]
pub struct PrototypeTriangle {
    /// Triangle vertices in deterministic winding order.
    pub vertices: [PrototypeVertex; 3],
}

impl PrototypeTriangle {
    fn normal(&self) -> PrototypeVertex {
        let edge_a = self.vertices[1].subtract(self.vertices[0]);
        let edge_b = self.vertices[2].subtract(self.vertices[0]);
        edge_a.cross(edge_b).normalize()
    }
}

/// Three-dimensional mesh vertex expressed in the mesh unit system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrototypeVertex {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate.
    pub z: f64,
}

impl PrototypeVertex {
    fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    fn normalize(self) -> Self {
        let magnitude = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if magnitude == 0.0 {
            return Self {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
        }

        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }
}

fn format_stl_scalar(value: f64) -> String {
    format!("{:.6}", canonical_stl_scalar(value))
}

fn canonical_stl_scalar(value: f64) -> f64 {
    let rounded = (value * 1_000_000.0).round() / 1_000_000.0;
    if rounded == -0.0 { 0.0 } else { rounded }
}

#[cfg(test)]
mod tests {
    use super::{
        PrototypeArtifact, PrototypeArtifactFormat, PrototypeLengthUnit, PrototypeTriangle,
        PrototypeTriangleMesh, PrototypeVertex,
    };

    #[test]
    fn creates_a_deterministic_stl_artifact_boundary() {
        let triangle = PrototypeTriangle {
            vertices: [
                PrototypeVertex {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                PrototypeVertex {
                    x: 10.0,
                    y: 0.0,
                    z: 0.0,
                },
                PrototypeVertex {
                    x: 0.0,
                    y: 10.0,
                    z: 0.0,
                },
            ],
        };
        let mesh = PrototypeTriangleMesh::new(vec![triangle.clone()]);
        let artifact = PrototypeArtifact::stl("spinner_body", mesh.clone());

        assert_eq!(artifact.name, "spinner_body");
        assert_eq!(artifact.format, PrototypeArtifactFormat::Stl);
        assert_eq!(artifact.mesh.units, PrototypeLengthUnit::Millimetres);
        assert_eq!(artifact.mesh.triangles, vec![triangle]);
        assert_eq!(artifact.mesh, mesh);
    }

    #[test]
    fn serializes_ascii_stl_deterministically() {
        let artifact = PrototypeArtifact::stl(
            "spinner_body",
            PrototypeTriangleMesh::new(vec![PrototypeTriangle {
                vertices: [
                    PrototypeVertex {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    PrototypeVertex {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    PrototypeVertex {
                        x: 0.0,
                        y: 10.0,
                        z: 0.0,
                    },
                ],
            }]),
        );

        assert_eq!(
            artifact.to_ascii_stl(),
            concat!(
                "solid spinner_body\n",
                "  facet normal 0.000000 0.000000 1.000000\n",
                "    outer loop\n",
                "      vertex 0.000000 0.000000 0.000000\n",
                "      vertex 10.000000 0.000000 0.000000\n",
                "      vertex 0.000000 10.000000 0.000000\n",
                "    endloop\n",
                "  endfacet\n",
                "endsolid spinner_body\n",
            )
        );
    }
}
