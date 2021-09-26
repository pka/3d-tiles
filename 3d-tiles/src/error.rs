/// Represents a tile loader error.
#[derive(Debug)]
pub enum Error {
    /// Io error occured.
    Io(::std::io::Error),
    /// Unsupported version.
    Version(u32),
    /// Wrong magic.
    Magic([u8; 4]),
    /// JSON decoding occured.
    Json(serde_json::error::Error),
}
