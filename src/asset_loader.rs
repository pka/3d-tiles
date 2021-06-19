use crate::b3dm::B3dm;
use crate::i3dm::I3dm;
use bevy::{
    asset::{AssetLoader, LoadContext},
    gltf::GltfLoader,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;
use std::ffi::OsStr;
use std::io::{Cursor, Read};

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "4c1bd5f9-8131-47ea-ac15-b6cf03b4473a"]
pub struct Tiles3dAsset;

#[derive(Default)]
pub struct Tiles3dAssetLoader;

impl AssetLoader for Tiles3dAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut reader = Cursor::new(bytes);
            match load_context.path().extension().and_then(OsStr::to_str) {
                Some("b3dm") => {
                    let _b3dm = B3dm::from_reader(&mut reader).unwrap();
                }
                Some("i3dm") => {
                    let _i3dm = I3dm::from_reader(&mut reader).unwrap();
                }
                _ => {
                    panic!("unexpected extension")
                }
            }
            let mut gltf_buf = Vec::new();
            reader.read_to_end(&mut gltf_buf)?;
            let gltf_loader = GltfLoader::default();
            gltf_loader.load(&gltf_buf, load_context).await?; // calls set_default_asset
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["b3dm", "i3dm"]
    }
}
