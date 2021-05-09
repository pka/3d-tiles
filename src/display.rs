use crate::asset_loader::{Cm3dTilesAsset, Cm3dTilesAssetLoader};
use bevy::gltf::Gltf;
use bevy::{pbr::AmbientLight, prelude::*};

pub fn display(tile_path: &str) {
    App::build()
        .insert_resource(Cm3dTilePath(tile_path.to_owned()))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_asset::<Cm3dTilesAsset>()
        .init_asset_loader::<Cm3dTilesAssetLoader>()
        .add_startup_system(setup.system())
        .add_system(rotator_system.system())
        .run();
}

pub struct Cm3dTilePath(String);

fn setup(mut commands: Commands, tile_path: Res<Cm3dTilePath>, asset_server: Res<AssetServer>) {
    let _gltf_handle: Handle<Gltf> = asset_server.load(tile_path.0.as_str());
    let scene_handle = asset_server.get_handle(format!("{}#Scene0", tile_path.0).as_str());
    commands.spawn_scene(scene_handle);
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 20.0)
            .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..Default::default()
    });
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(3.0, 5.0, 3.0),
            ..Default::default()
        })
        .insert(Rotates);
}

/// this component indicates what entities should rotate
struct Rotates;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_y(
            (4.0 * std::f32::consts::PI / 20.0) * time.delta_seconds(),
        )) * *transform;
    }
}
