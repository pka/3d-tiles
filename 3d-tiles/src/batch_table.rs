use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

/// The Batch Table contains per-model application-specific properties.
// <https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/BatchTable/README.md>
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
        use Error::Io;
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

/// A set of properties defining application-specific metadata for features in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchTableJson {
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
    Array(Vec<serde_json::Value>),
    BinaryBodyReference(BinaryBodyReference),
}

/// An object defining the reference to a section of the binary body of the batch table where
/// the property values are stored if not defined directly in the JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryBodyReference {
    /// The offset into the buffer in bytes.
    #[serde(rename = "byteOffset")]
    pub byte_offset: usize,
    /// The datatype of components in the property.
    #[serde(rename = "componentType")]
    pub component_type: ComponentType,
    /// Specifies if the property is a scalar or vector.
    #[serde(rename = "type")]
    pub property_type: Type,
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
