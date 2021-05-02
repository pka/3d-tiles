use crate::batch_table::BatchTable;
use crate::error::Error;
use crate::feature_table::{GlobalPropertyCartesian3, GlobalPropertyScalar, Property};
use byteorder::{LittleEndian, ReadBytesExt};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/// Batched 3D Model.
/// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/Batched3DModel/README.md
#[derive(Debug)]
pub struct B3dm {
    pub header: Header,
    pub feature_table: FeatureTable,
    pub batch_table: BatchTable,
    // Binary GlTF
}

/// The header section of a .b3dm file.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct FeatureTable {
    pub json: BatchedFeatureTable,
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
        let json: BatchedFeatureTable = serde_json::from_slice(&buf).map_err(Error::Json)?;
        let mut body = vec![0; binary_byte_length as usize];
        reader.read_exact(&mut body).map_err(Io)?;
        Ok(FeatureTable { json, body })
    }
}

/// A set of Batched 3D Model semantics that contain additional information about features in
/// a tile.
///
/// A set of semantics containing per-tile and per-feature values defining the position and
/// appearance properties for features in a tile.
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchedFeatureTable {
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

    #[serde(flatten)]
    pub properties: HashMap<String, Property>,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
}

impl B3dm {
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
        Ok(B3dm {
            header,
            feature_table,
            batch_table,
        })
    }
}

/// Read b3dm file and extract binary GlTF
pub fn extract_glb(path: &str) -> Result<B3dm, Error> {
    use self::Error::Io;
    let file = File::open(path).map_err(Io)?;
    let mut reader = BufReader::new(file);
    let b3dm = B3dm::from_reader(&mut reader)?;
    dbg!(&b3dm.feature_table.json);
    dbg!(&b3dm.batch_table.json);

    let dest = Path::new(path).with_extension("glb");
    println!("Writing {:?}", &dest);
    let mut file = File::create(dest).map_err(Io)?;
    io::copy(&mut reader, &mut file).map_err(Io)?;
    Ok(b3dm)
}
