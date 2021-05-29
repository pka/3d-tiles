use crate::asset_loader::{Cm3dTilesAsset, Cm3dTilesAssetLoader};
use crate::batch_table::BatchTable;
use crate::pnts::Pnts;
use bevy::gltf::Gltf;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{pbr::AmbientLight, prelude::*};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Read};

pub fn display_gltf(tile_path: &str) {
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
        .add_startup_system(setup_gltf.system())
        .add_system(rotator_system.system())
        .run();
}

pub fn display_pnts(tile_path: &str) {
    App::build()
        .insert_resource(Cm3dTilePath(tile_path.to_owned()))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_pnts.system())
        .run();
}

pub struct Cm3dTilePath(String);

fn setup_gltf(
    mut commands: Commands,
    tile_path: Res<Cm3dTilePath>,
    asset_server: Res<AssetServer>,
) {
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

fn setup_pnts(
    mut commands: Commands,
    tile_path: Res<Cm3dTilePath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let file = File::open(tile_path.0.as_str()).unwrap();
    let mut reader = BufReader::new(file);
    let pnts = Pnts::from_reader(&mut reader).unwrap();
    dbg!(&pnts.feature_table.json);

    let points_length = pnts.feature_table.json.points_length as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(points_length);
    for _ in 0..points_length {
        positions.push([
            reader.read_f32::<LittleEndian>().unwrap(),
            reader.read_f32::<LittleEndian>().unwrap(),
            reader.read_f32::<LittleEndian>().unwrap(),
        ]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::PointList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![0.0; points_length]);

    // Skip remaining feature data
    let mut body = vec![
        0;
        pnts.header.feature_table_binary_byte_length as usize
            - (points_length * std::mem::size_of::<f32>() * 3)
    ];
    reader.read_exact(&mut body).unwrap();

    let batch_table = BatchTable::from_reader(
        &mut reader,
        pnts.header.batch_table_json_byte_length,
        pnts.header.batch_table_binary_byte_length,
    )
    .unwrap();
    dbg!(&batch_table.json);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
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
