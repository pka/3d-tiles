use crate::error::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

/// A 3D Tiles tileset.
///
/// <https://github.com/CesiumGS/3d-tiles/tree/1.0/specification#tileset-json>
#[derive(Debug, Serialize, Deserialize)]
pub struct Tileset {
    /// Metadata about the entire tileset.
    pub asset: Asset,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Names of 3D Tiles extensions required to properly load this tileset.
    #[serde(rename = "extensionsRequired")]
    pub extensions_required: Option<Vec<String>>,
    /// Names of 3D Tiles extensions used somewhere in this tileset.
    #[serde(rename = "extensionsUsed")]
    pub extensions_used: Option<Vec<String>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
    /// The error, in meters, introduced if this tileset is not rendered. At runtime, the
    /// geometric error is used to compute screen space error (SSE), i.e., the error measured in
    /// pixels.
    #[serde(rename = "geometricError")]
    pub geometric_error: f64,
    /// A dictionary object of metadata about per-feature properties.
    pub properties: Option<PropertiesUnion>,
    /// The root tile.
    pub root: Tile,
}

/// Metadata about the entire tileset.
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    pub extras: Option<serde_json::Value>,
    /// Application-specific version of this tileset, e.g., for when an existing tileset is
    /// updated.
    #[serde(rename = "tilesetVersion")]
    pub tileset_version: Option<String>,
    /// The 3D Tiles version.  The version defines the JSON schema for the tileset JSON and the
    /// base set of tile formats.
    pub version: String,
}

/// A dictionary object of metadata about per-feature properties.
#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    pub extras: Option<serde_json::Value>,
    /// The maximum value of this property of all the features in the tileset.
    pub maximum: f64,
    /// The minimum value of this property of all the features in the tileset.
    pub minimum: f64,
}

/// A dictionary object of metadata about per-feature properties.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertiesUnion {
    AnythingArray(Vec<Option<serde_json::Value>>),
    Bool(bool),
    Double(f64),
    Integer(i64),
    PropertiesMap(HashMap<String, Properties>),
    String(String),
}

/// A tile in a 3D Tiles tileset.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    /// The bounding volume that encloses the tile.
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: BoundingVolume,
    /// An array of objects that define child tiles. Each child tile content is fully enclosed by
    /// its parent tile's bounding volume and, generally, has a geometricError less than its
    /// parent tile's geometricError. For leaf tiles, the length of this array is zero, and
    /// children may not be defined.
    pub children: Option<Vec<Tile>>,
    /// Metadata about the tile's content and a link to the content. When this is omitted the
    /// tile is just used for culling. This is required for leaf tiles.
    pub content: Option<TileContent>,
    /// Dictionary object with extension-specific objects.
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    /// Application-specific data.
    pub extras: Option<serde_json::Value>,
    /// The error, in meters, introduced if this tile is rendered and its children are not. At
    /// runtime, the geometric error is used to compute screen space error (SSE), i.e., the error
    /// measured in pixels.
    #[serde(rename = "geometricError")]
    pub geometric_error: f64,
    /// Specifies if additive or replacement refinement is used when traversing the tileset for
    /// rendering.  This property is required for the root tile of a tileset; it is optional for
    /// all other tiles.  The default is to inherit from the parent tile.
    pub refine: Option<Refine>,
    /// A floating-point 4x4 affine transformation matrix, stored in column-major order, that
    /// transforms the tile's content--i.e., its features as well as content.boundingVolume,
    /// boundingVolume, and viewerRequestVolume--from the tile's local coordinate system to the
    /// parent tile's coordinate system, or, in the case of a root tile, from the tile's local
    /// coordinate system to the tileset's coordinate system.  transform does not apply to
    /// geometricError, nor does it apply any volume property when the volume is a region,
    /// defined in EPSG:4979 coordinates.
    pub transform: Option<Vec<f32>>,
    /// Optional bounding volume that defines the volume the viewer must be inside of before the
    /// tile's content will be requested and before the tile will be refined based on
    /// geometricError.
    #[serde(rename = "viewerRequestVolume")]
    pub viewer_request_volume: Option<BoundingVolume>,
}

/// The bounding volume that encloses the tile.
///
/// An optional bounding volume that tightly encloses just the tile's content.
/// tile.boundingVolume provides spatial coherence and tile.content.boundingVolume enables
/// tight view frustum culling. When this is omitted, tile.boundingVolume is used.
///
/// Optional bounding volume that defines the volume the viewer must be inside of before the
/// tile's content will be requested and before the tile will be refined based on
/// geometricError.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingVolume {
    /// An array of 12 numbers that define an oriented bounding box.  The first three elements
    /// define the x, y, and z values for the center of the box.  The next three elements (with
    /// indices 3, 4, and 5) define the x axis direction and half-length.  The next three
    /// elements (indices 6, 7, and 8) define the y axis direction and half-length.  The last
    /// three elements (indices 9, 10, and 11) define the z axis direction and half-length.
    #[serde(rename = "box")]
    pub bounding_volume_box: Option<Vec<f64>>,
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    pub extras: Option<serde_json::Value>,
    /// An array of six numbers that define a bounding geographic region in EPSG:4979 coordinates
    /// with the order [west, south, east, north, minimum height, maximum height]. Longitudes and
    /// latitudes are in radians, and heights are in meters above (or below) the WGS84 ellipsoid.
    pub region: Option<Vec<f64>>,
    /// An array of four numbers that define a bounding sphere.  The first three elements define
    /// the x, y, and z values for the center of the sphere.  The last element (with index 3)
    /// defines the radius in meters.
    pub sphere: Option<Vec<f64>>,
}

/// Metadata about the tile's content and a link to the content. When this is omitted the
/// tile is just used for culling. This is required for leaf tiles.
#[derive(Debug, Serialize, Deserialize)]
pub struct TileContent {
    /// An optional bounding volume that tightly encloses just the tile's content.
    /// tile.boundingVolume provides spatial coherence and tile.content.boundingVolume enables
    /// tight view frustum culling. When this is omitted, tile.boundingVolume is used.
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: Option<BoundingVolume>,
    pub extensions: Option<HashMap<String, HashMap<String, Option<serde_json::Value>>>>,
    pub extras: Option<serde_json::Value>,
    /// A uri that points to the tile's content. When the uri is relative, it is relative to the
    /// referring tileset JSON file.
    pub uri: String,
}

/// Specifies if additive or replacement refinement is used when traversing the tileset for
/// rendering.  This property is required for the root tile of a tileset; it is optional for
/// all other tiles.  The default is to inherit from the parent tile.
#[derive(Debug, Serialize, Deserialize)]
pub enum Refine {
    #[serde(rename = "ADD")]
    Add,
    #[serde(rename = "REPLACE")]
    Replace,
}

impl Tileset {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let tileset: Tileset = serde_json::from_reader(reader).map_err(Error::Json)?;
        Ok(tileset)
    }
}
