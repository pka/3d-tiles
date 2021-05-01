use crate::common::{GlobalPropertyScalar, GlobalPropertyCartesian3, BatchTable, BinaryBodyReference, Error};
use byteorder::{LittleEndian, ReadBytesExt};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// Instanced 3D Model.
/// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/Instanced3DModel/README.md
#[derive(Debug)]
pub struct I3dm {
    pub header: Header,
    pub feature_table: FeatureTable,
    pub batch_table: BatchTable,
    // GlTF
}

/// The header section of a .i3dm file.
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    /// Must be `b"i3dm"`. This can be used to identify the content as an Instanced 3D Model tile.
    pub magic: [u8; 4],
    /// The version of the Instanced 3D Model format. It is currently `1`.
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
    /// Indicates the format of the glTF field of the body. 0 indicates it is a uri, 1 indicates it is embedded binary glTF. See the glTF section below.
    pub gltf_format: u32,
}

impl Header {
    fn from_reader<R: Read>(mut reader: R) -> Result<Self, Error> {
        use crate::common::Error::Io;
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).map_err(Io)?;
        if &magic == b"i3dm" {
            Ok(Self {
                magic,
                version: reader.read_u32::<LittleEndian>().map_err(Io)?,
                byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                feature_table_json_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                feature_table_binary_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                batch_table_json_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                batch_table_binary_byte_length: reader.read_u32::<LittleEndian>().map_err(Io)?,
                gltf_format: reader.read_u32::<LittleEndian>().map_err(Io)?,
            })
        } else {
            Err(Error::Magic(magic))
        }
    }
}

/// A Feature Table is a component of a tile's binary body and describes position and appearance properties required to render each feature in a tile.
// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/FeatureTable/README.md
#[derive(Debug)]
pub struct FeatureTable {
    pub json: InstancedFeatureTable,
    pub body: Vec<u8>,
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
        let json: InstancedFeatureTable = serde_json::from_slice(&buf).map_err(Error::Json)?;
        let mut body = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut body).map_err(Io)?;
        Ok(FeatureTable { json, body })
    }
}

/// A set of semantics containing per-tile and per-feature values defining the position and
/// appearance properties for features in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstancedFeatureTable {
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "BATCH_ID")]
    pub batch_id: Option<BinaryBodyReference>,
    /// A `GlobalPropertyBoolean` object defining a boolean property for all features. See the
    /// corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "EAST_NORTH_UP")]
    pub east_north_up: Option<bool>,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
    /// A `GlobalPropertyScalar` object defining a numeric property for all features. See the
    /// corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "INSTANCES_LENGTH")]
    pub instances_length: GlobalPropertyScalar,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "NORMAL_RIGHT")]
    pub normal_right: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "NORMAL_RIGHT_OCT32P")]
    pub normal_right_oct32_p: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "NORMAL_UP")]
    pub normal_up: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "NORMAL_UP_OCT32P")]
    pub normal_up_oct32_p: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "POSITION")]
    pub position: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "POSITION_QUANTIZED")]
    pub position_quantized: Option<BinaryBodyReference>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// features. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "QUANTIZED_VOLUME_OFFSET")]
    pub quantized_volume_offset: Option<GlobalPropertyCartesian3>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// features. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "QUANTIZED_VOLUME_SCALE")]
    pub quantized_volume_scale: Option<GlobalPropertyCartesian3>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// features. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "RTC_CENTER")]
    pub rtc_center: Option<GlobalPropertyCartesian3>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "SCALE")]
    pub scale: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](/specification/TileFormats/Instanced3DModel/README.md#semantics).
    #[serde(rename = "SCALE_NON_UNIFORM")]
    pub scale_non_uniform: Option<BinaryBodyReference>,
}

impl I3dm {
    fn from_reader<R: Read>(mut reader: R) -> Result<Self, Error> {
        let header = Header::from_reader(&mut reader)?;
        if header.version != 1 {
            return Err(Error::Version(header.version));
        }
        let feature_table = FeatureTable::from_reader(
            &mut reader,
            header.feature_table_json_byte_length,
            header.feature_table_binary_byte_length,
        )?;
        let batch_table = BatchTable::from_reader(
            &mut reader,
            header.batch_table_json_byte_length,
            header.batch_table_binary_byte_length,
        )?;
        Ok(I3dm {
            header,
            feature_table,
            batch_table,
        })
    }
}

/// Read i3dm file and extract binary GlTF
pub fn extract_gltf(path: &str) -> Result<I3dm, Error> {
    let file = File::open(path).map_err(Error::Io)?;
    let mut reader = BufReader::new(file);
    let i3dm = I3dm::from_reader(&mut reader)?;
    dbg!(&i3dm.feature_table.json);
    dbg!(&i3dm.batch_table.json);

    if i3dm.header.gltf_format == 0 {
        let mut url = String::new();
        reader.read_to_string(&mut url);
        dbg!(&url);
    } else if i3dm.header.gltf_format == 1 {
        let dest = Path::new(path).with_extension("glb");
        println!("Writing {:?}", &dest);
        let mut file = File::create(dest).map_err(Error::Io)?;
        io::copy(&mut reader, &mut file).map_err(Error::Io)?;
    }
    Ok(i3dm)
}
