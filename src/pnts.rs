use crate::batch_table::BatchTable;
use crate::error::Error;
use crate::feature_table::{
    BinaryBodyReference, FluffyGlobalPropertyScalar, GlobalPropertyCartesian3,
    GlobalPropertyCartesian4, Property, PurpleGlobalPropertyScalar,
};
use byteorder::{LittleEndian, ReadBytesExt};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

/// Point Clouds.
/// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/PointCloud/README.md
#[derive(Debug)]
pub struct Pnts {
    pub header: Header,
    pub feature_table: FeatureTable,
    pub batch_table: BatchTable,
}

/// The header section of a .pnts file.
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    /// Must be `b"pnts"`. This can be used to identify the content as a Point Cloud tile.
    pub magic: [u8; 4],
    /// The version of the Point Cloud format. It is currently `1`.
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
        if &magic == b"pnts" {
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
#[derive(Debug)]
pub struct FeatureTable {
    pub json: PntsTable,
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
        // dbg!(&std::str::from_utf8(&buf));
        let json: PntsTable = serde_json::from_slice(&buf).map_err(Error::Json)?;
        let mut body = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut body).map_err(Io)?;
        Ok(FeatureTable { json, body })
    }
}

/// A set of Point Cloud semantics that contains values defining the position and appearance
/// properties for points in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct PntsTable {
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "BATCH_ID")]
    pub batch_id: Option<BinaryBodyReference>,
    /// A `GlobalPropertyScalar` object defining a numeric property for all points. See the
    /// corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "BATCH_LENGTH")]
    pub batch_length: Option<PurpleGlobalPropertyScalar>,
    /// A `GlobalPropertyCartesian4` object defining a 4-component numeric property for all
    /// points. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "CONSTANT_RGBA")]
    pub constant_rgba: Option<GlobalPropertyCartesian4>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "NORMAL")]
    pub normal: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "NORMAL_OCT16P")]
    pub normal_oct16_p: Option<BinaryBodyReference>,
    /// A `GlobalPropertyScalar` object defining a numeric property for all points. See the
    /// corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "POINTS_LENGTH")]
    pub points_length: FluffyGlobalPropertyScalar,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "POSITION")]
    pub position: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "POSITION_QUANTIZED")]
    pub position_quantized: Option<BinaryBodyReference>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// points. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "QUANTIZED_VOLUME_OFFSET")]
    pub quantized_volume_offset: Option<GlobalPropertyCartesian3>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// points. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "QUANTIZED_VOLUME_SCALE")]
    pub quantized_volume_scale: Option<GlobalPropertyCartesian3>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "RGB")]
    pub rgb: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "RGB565")]
    pub rgb565: Option<BinaryBodyReference>,
    /// A `BinaryBodyReference` object defining the reference to a section of the binary body
    /// where the property values are stored. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "RGBA")]
    pub rgba: Option<BinaryBodyReference>,
    /// A `GlobalPropertyCartesian3` object defining a 3-component numeric property for all
    /// points. See the corresponding property semantic in
    /// [Semantics](https://github.com/CesiumGS/3d-tiles/blob/1.0/specification/TileFormats/PointCloud/README.md#semantics).
    #[serde(rename = "RTC_CENTER")]
    pub rtc_center: Option<GlobalPropertyCartesian3>,

    #[serde(flatten)]
    pub properties: HashMap<String, Property>,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
}

impl Pnts {
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
        Ok(Pnts {
            header,
            feature_table,
            batch_table,
        })
    }
}

/// Read pnts file
pub fn extract(path: &str) -> Result<Pnts, Error> {
    use self::Error::Io;
    let file = File::open(path).map_err(Io)?;
    let mut reader = BufReader::new(file);
    let pnts = Pnts::from_reader(&mut reader)?;
    dbg!(&pnts.feature_table.json);
    dbg!(&pnts.batch_table.json);
    Ok(pnts)
}
