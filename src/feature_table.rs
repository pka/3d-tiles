use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// A set of semantics containing per-tile and per-feature values defining the position and
/// appearance properties for features in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureTable {
    #[serde(flatten)]
    pub properties: HashMap<String, Property>,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
}

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
    pub byte_offset: f64,
    /// The datatype of components in the property. This is defined only if the semantic allows
    /// for overriding the implicit component type. These cases are specified in each tile format.
    #[serde(rename = "componentType")]
    pub component_type: Option<ComponentType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyScalarClass {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyCartesian3Class {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: f64,
}

/// A `GlobalPropertyScalar` object defining a numeric property for all features. See the
/// corresponding property semantic in
/// [Semantics](/specification/TileFormats/Batched3DModel/README.md#semantics).
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
/// [Semantics](/specification/TileFormats/Batched3DModel/README.md#semantics).
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
