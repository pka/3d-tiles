// https://github.com/CesiumGS/3d-tiles/blob/master/specification/TileFormats/Batched3DModel/README.md

use byteorder::{LittleEndian, ReadBytesExt};
use std::fs;
use std::io;

/// Represents a b3dm loader error.
#[derive(Debug)]
pub enum Error {
    /// Io error occured.
    Io(::std::io::Error),
    /// Unsupported version.
    Version(u32),
    /// Magic says that file is not b3dm.
    Magic([u8; 4]),
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
    fn from_reader<R: io::Read>(mut reader: R) -> Result<Self, Error> {
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

pub fn extract(path: &str) -> Result<(), Error> {
    use self::Error::Io;
    let file = fs::File::open(path).map_err(Io)?;
    let mut reader = io::BufReader::new(file);
    let header = Header::from_reader(&mut reader)?;
    dbg!(header);
    Ok(())
}
