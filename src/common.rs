use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

/// Represents a b3dm loader error.
#[derive(Debug)]
pub enum Error {
    /// Io error occured.
    Io(::std::io::Error),
    /// Unsupported version.
    Version(u32),
    /// Magic says that file is not b3dm.
    Magic([u8; 4]),
    /// JSON decoding occured.
    Json(serde_json::error::Error),
}

/// The Batch Table contains per-model application-specific properties.
// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/BatchTable/README.md
#[derive(Debug)]
pub struct BatchTable {
    pub json: Option<BatchTableJson>,
    pub body: Vec<u8>,
}

impl BatchTable {
    pub fn from_reader<R: Read>(
        mut reader: R,
        json_byte_length: u32,
        binary_byte_length: u32,
    ) -> Result<Self, Error> {
        use self::Error::Io;
        let json = if json_byte_length > 0 {
            let mut buf = vec![0; json_byte_length as usize];
            reader.read_exact(&mut buf).map_err(Io)?;
            dbg!(&std::str::from_utf8(&buf));
            let json: BatchTableJson = serde_json::from_slice(&buf).map_err(Error::Json)?;
            Some(json)
        } else {
            None
        };
        let mut body = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut body).map_err(Io)?;
        Ok(BatchTable { json, body })
    }
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

/// A set of properties defining application-specific metadata for features in a tile.
pub type BatchTableJson = HashMap<String, Property>;

/// An object defining the reference to a section of the binary body of the batch table where
/// the property values are stored if not defined directly in the JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryBodyReference {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: f64,
    /// The datatype of components in the property.
    #[serde(rename = "componentType")]
    pub component_type: Option<ComponentType>,
    /// Specifies if the property is a scalar or vector.
    #[serde(rename = "type")]
    pub binary_body_reference_type: Option<Type>,
}


/// A user-defined property which specifies per-feature application-specific metadata in a
/// tile. Values either can be defined directly in the JSON as an array, or can refer to
/// sections in the binary body with a `BinaryBodyReference` object.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    Array(Vec<serde_json::Value>),
    BinaryBodyReference(BinaryBodyReference),
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
