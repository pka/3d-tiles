// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/Batched3DModel/README.md

use byteorder::{LittleEndian, ReadBytesExt};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};

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

/// The header section of a .b3dm file.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Header {
    /// Must be `b"b3dm"`. This can be used to identify the content as a Batched 3D Model tile.
    pub magic: [u8; 4],
    /// The version of the Batched 3D Model format. It is currently `1`.
    pub version: u32,
    /// The length of the entire tile, including the header, in bytes.
    pub byte_length: u32,
    /// The length of the Feature Table JSON section in bytes.
    pub feature_table_json_byte_length: u32,
    /// The length of the Feature Table binary section in bytes.
    pub feature_table_binary_byte_length: u32,
    /// The length of the Batch Table JSON section in bytes. Zero indicates there is no Batch Table.
    pub batch_table_json_byte_length: u32,
    /// The length of the Batch Table binary section in bytes. If `batchTableJSONByteLength` is zero, this will also be zero.
    pub batch_table_binary_byte_length: u32,
}

impl Header {
    fn from_reader<R: Read>(mut reader: R) -> Result<Self, Error> {
        use self::Error::Io;
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).map_err(Io)?;
        // We only validate magic as we don't care for version and length of
        // contents, the caller does.  Let them decide what to do next with
        // regard to version and length.
        if &magic == b"b3dm" {
            Ok(Self {
                magic,
                version: reader.read_u32::<LittleEndian>().map_err(Io)?,
                byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                feature_table_json_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                feature_table_binary_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                batch_table_json_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                batch_table_binary_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
            })
        } else {
            Err(Error::Magic(magic))
        }
    }
}

/// A Feature Table is a component of a tile's binary body and describes position and appearance properties required to render each feature in a tile.
// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/FeatureTable/README.md
pub struct FeatureTable {
    json: FeatureTableJson,
    // body: Vec<u8>,
}

impl FeatureTable {
    fn from_reader<R: Read>(
        mut reader: R,
        json_byte_length: u32,
        binary_byte_length: u32,
    ) -> Result<Self, Error> {
        use self::Error::Io;
        let mut buf = vec![0; json_byte_length as usize];
        reader.read_exact(&mut buf).map_err(Io)?;
        dbg!(&std::str::from_utf8(&buf));
        let json: FeatureTableJson = serde_json::from_slice(&buf).map_err(Error::Json)?;
        let mut buf = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut buf).map_err(Io)?;
        Ok(FeatureTable { json })
    }
}

/// The Batch Table contains per-model application-specific properties.
// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/BatchTable/README.md
pub struct BatchTable {
    json: Option<BatchTableJson>,
    // body: Vec<u8>,
}

impl BatchTable {
    fn from_reader<R: Read>(
        mut reader: R,
        json_byte_length: u32,
        binary_byte_length: u32,
    ) -> Result<Self, Error> {
        use self::Error::Io;
        let json = if json_byte_length > 0 {
            let mut buf = vec![0; json_byte_length as usize];
            reader.read_exact(&mut buf).map_err(Io)?;
            let json: BatchTableJson = serde_json::from_slice(&buf).map_err(Error::Json)?;
            Some(json)
        } else {
            None
        };
        let mut buf = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut buf).map_err(Io)?;
        Ok(BatchTable { json })
    }
}

/// A set of Batched 3D Model semantics that contain additional information about features in
/// a tile.
///
/// A set of semantics containing per-tile and per-feature values defining the position and
/// appearance properties for features in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureTableJson {
    /// A `GlobalPropertyScalar` object defining a numeric property for all features. See the
    /// corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Batched3DModel/README.md#semantics).
    #[serde(rename = "BATCH_LENGTH")]
    pub batch_length: GlobalPropertyScalar,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// features. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Batched3DModel/README.md#semantics).
    #[serde(rename = "RTC_CENTER")]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
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
    pub component_type: ComponentType,
    /// Specifies if the property is a scalar or vector.
    #[serde(rename = "type")]
    pub binary_body_reference_type: Type,
}

/// A user-defined property which specifies per-feature application-specific metadata in a
/// tile. Values either can be defined directly in the JSON as an array, or can refer to
/// sections in the binary body with a `BinaryBodyReference` object.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    AnythingMapArray(Vec<HashMap<String, Option<serde_json::Value>>>),
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

pub fn extract(path: &str) -> Result<(), Error> {
    use self::Error::Io;
    let file = File::open(path).map_err(Io)?;
    let mut reader = BufReader::new(file);
    let header = Header::from_reader(&mut reader)?;
    if header.version != 1 {
        return Err(Error::Version(header.version));
    }
    let feature_table = FeatureTable::from_reader(
        &mut reader,
        header.feature_table_json_byte_length,
        header.feature_table_binary_byte_length,
    )?;
    dbg!(&feature_table.json);
    let batch_table = BatchTable::from_reader(
        &mut reader,
        header.batch_table_json_byte_length,
        header.batch_table_binary_byte_length,
    )?;
    dbg!(&batch_table.json);

    let mut file = File::create("model.gltf").map_err(Io)?;
    io::copy(&mut reader, &mut file).map_err(Io)?;
    Ok(())
}
