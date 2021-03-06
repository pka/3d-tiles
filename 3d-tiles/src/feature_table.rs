use serde_derive::{Deserialize, Serialize};

/// A user-defined property which specifies per-feature application-specific metadata in a
/// tile. Values either can be defined directly in the JSON as an array, or can refer to
/// sections in the binary body with a `BinaryBodyReference` object.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    BinaryBodyReference(BinaryBodyReference),
    Double(f64),
    DoubleArray(Vec<f64>),
}

/// An object defining the reference to a section of the binary body of the features table
/// where the property values are stored if not defined directly in the JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryBodyReference {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
    /// The datatype of components in the property. This is defined only if the semantic allows
    /// for overriding the implicit component type. These cases are specified in each tile format.
    #[serde(rename = "componentType")]
    pub component_type: Option<ComponentType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyScalarClass {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyCartesian3Class {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
}

/// A `GlobalPropertyScalar` object defining a numeric property for all features. See the
/// corresponding property semantic in
/// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/Batched3DModel/README.md#semantics).
///
/// An object defining a global numeric property value for all features.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyScalar {
    Double(f64),
    DoubleArray(Vec<f64>),
    GlobalPropertyScalarClass(GlobalPropertyScalarClass),
}

/// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
/// features. See the corresponding property semantic in
/// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/Batched3DModel/README.md#semantics).
///
/// An object defining a global 3-component numeric property values for all features.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian3 {
    DoubleArray(Vec<f64>),
    GlobalPropertyCartesian3Class(GlobalPropertyCartesian3Class),
}

/// Specifies if the property is a scalar or vector.
#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "SCALAR")]
    Scalar,
    #[serde(rename = "VEC2")]
    Vec2,
    #[serde(rename = "VEC3")]
    Vec3,
    #[serde(rename = "VEC4")]
    Vec4,
}

/// The datatype of components in the property.
#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentType {
    #[serde(rename = "BYTE")]
    Byte,
    #[serde(rename = "DOUBLE")]
    Double,
    #[serde(rename = "FLOAT")]
    Float,
    #[serde(rename = "INT")]
    Int,
    #[serde(rename = "SHORT")]
    Short,
    #[serde(rename = "UNSIGNED_BYTE")]
    UnsignedByte,
    #[serde(rename = "UNSIGNED_INT")]
    UnsignedInt,
    #[serde(rename = "UNSIGNED_SHORT")]
    UnsignedShort,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyCartesian4Class {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
}

/// A `GlobalPropertyScalar` object defining a numeric property for all points. See the
/// corresponding property semantic in
/// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
///
/// An object defining a global numeric property value for all features.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PurpleGlobalPropertyScalar {
    Double(f64),
    DoubleArray(Vec<f64>),
    GlobalPropertyScalar(GlobalPropertyScalar),
}

/// A `GlobalPropertyCartesian4` object defining a 4-component numeric property for all
/// points. See the corresponding property semantic in
/// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
///
/// An object defining a global 4-component numeric property values for all features.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobalPropertyCartesian4 {
    DoubleArray(Vec<f64>),
    GlobalPropertyCartesian4Class(GlobalPropertyCartesian4Class),
}

/// A `GlobalPropertyScalar` object defining a numeric property for all points. See the
/// corresponding property semantic in
/// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
///
/// An object defining a global numeric property value for all features.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FluffyGlobalPropertyScalar {
    Double(f64),
    DoubleArray(Vec<f64>),
    GlobalPropertyScalar(GlobalPropertyScalar),
}
