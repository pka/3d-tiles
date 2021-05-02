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
