use crate::asset_loader::{Tiles3dAsset, Tiles3dAssetLoader};
use bevy::gltf::Gltf;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_inspector_egui::WorldInspectorPlugin;
use byteorder::{LittleEndian, ReadBytesExt};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use tiles3d::b3dm::B3dm;
use tiles3d::batch_table::BatchTable;
use tiles3d::i3dm::I3dm;
use tiles3d::pnts::Pnts;
use tiles3d::tileset::{Tile, Tileset};

pub fn view_tileset(tileset_path: &str) {
    let mut app = App::build();
    init_viewer(&mut app);
    view_tileset_content(&mut app, tileset_path);
    app.run();
}

fn read_tileset_json(tileset_path: &str) -> Tileset {
    let mut file = File::open(tileset_path).expect(&format!("Couldn't open file {}", tileset_path));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let tileset: Tileset = serde_json::from_slice(&buf).expect("Invalid Tileset JSON");
    dbg!(&tileset);
    tileset
}

fn view_tileset_content(app: &mut AppBuilder, tileset_path: &str) {
    let tileset = read_tileset_json(tileset_path);
    let mut tile = &tileset.root;
    if let Some(ref bounding_volume_box) = tile.bounding_volume.bounding_volume_box {
        app.insert_resource(BoundingVolumeBox(bounding_volume_box.clone()))
            .add_startup_system(setup_bounding_volume.system());
    }
    if tile.content.is_some() {
        view_tile(app, tileset_path, &tile);
    }
    while tile.content.is_none() {
        if let Some(ref children) = tile.children {
            for child in children {
                tile = child;
                if tile.content.is_some() {
                    view_tile(app, tileset_path, &tile);
                }
            }
        }
    }
}

/// File path for tile in tileset
fn tile_fn(tileset_path: &str, tile_uri: &str) -> String {
    let mut tile_path = Path::new(&tileset_path).parent().unwrap().to_path_buf();
    tile_path.push(&tile_uri);
    let tile_fn = tile_path.into_os_string();
    let tile_fn = tile_fn.to_str().expect("Invalid file name");
    tile_fn.to_string()
}

fn view_tile(app: &mut AppBuilder, tileset_path: &str, tile: &Tile) {
    let tile_uri = &tile.content.as_ref().expect("Tile content missing").uri;
    let tile_fn = tile_fn(tileset_path, &tile_uri);
    dbg!(&tile_fn);
    let file = File::open(&tile_fn).expect(&format!("Couldn't open file {}", &tile_fn));
    let mut reader = BufReader::new(file);

    match Path::new(&tile_uri).extension().and_then(OsStr::to_str) {
        Some("b3dm") => {
            let b3dm = B3dm::from_reader(&mut reader).expect("Invalid b3dm");
            // dbg!(&b3dm.feature_table.json);
            // dbg!(&b3dm.batch_table.json);
            if b3dm.feature_table.json.rtc_center.is_some() {
                println!(
                    "TODO: add transformation for rtc_center {:?}",
                    b3dm.feature_table.json.rtc_center
                );
            }
            view_gltf_from_reader(app, tile.transform.clone(), &mut reader);
        }
        Some("i3dm") => {
            let i3dm = I3dm::from_reader(&mut reader).expect("Invalid i3dm");
            // dbg!(&i3dm.feature_table.json);
            // dbg!(&i3dm.batch_table.json);
            if i3dm.feature_table.json.rtc_center.is_some() {
                println!(
                    "TODO: add transformation for rtc_center {:?}",
                    i3dm.feature_table.json.rtc_center
                );
            }

            if i3dm.header.gltf_format == 0 {
                let mut url = String::new();
                reader.read_to_string(&mut url).unwrap();
                dbg!(&url); // TODO
            } else if i3dm.header.gltf_format == 1 {
                view_gltf_from_reader(app, tile.transform.clone(), &mut reader);
            }
        }
        Some("pnts") => {
            view_pnts(app, tile.transform.clone(), &tile_fn);
        }
        Some("json") => {
            view_tileset_content(app, &tile_fn);
        }
        _ => {
            println!("Unknown file extension");
        }
    }
}

fn view_gltf_from_reader<R: Read>(
    app: &mut AppBuilder,
    transform: Option<Vec<f32>>,
    mut reader: R,
) {
    // Write glTF into file
    let mut file = tempfile::Builder::new()
        .prefix("tile_")
        .suffix(".glb")
        .tempfile()
        .expect("Couldn't create tempfile");
    io::copy(&mut reader, &mut file).unwrap();
    let (_file, path) = file.keep().expect("tempfile keep failed");
    let gltf_fn = path.to_str().expect("Invalid file name");
    view_gltf(app, transform, &gltf_fn);
}

pub fn init_viewer(app: &mut AppBuilder) {
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(setup_camera.system())
        .add_system(rotator_system.system());

    // glTF viewer
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    })
    .add_asset::<Tiles3dAsset>()
    .init_asset_loader::<Tiles3dAssetLoader>()
    .add_startup_system(setup_gltf.system());

    // Points viewer
    app.add_startup_system(setup_pnts.system());
}

pub fn view_gltf(app: &mut AppBuilder, transform: Option<Vec<f32>>, tile_path: &str) {
    app.world_mut().spawn().insert(GltfTile {
        path: tile_path.to_owned(),
        transform,
    });
}

pub fn view_pnts(app: &mut AppBuilder, transform: Option<Vec<f32>>, tile_path: &str) {
    app.world_mut().spawn().insert(PntsTile {
        path: tile_path.to_owned(),
        transform,
    });
}

struct GltfTile {
    path: String,
    transform: Option<Vec<f32>>,
}

fn setup_gltf(mut commands: Commands, query: Query<&GltfTile>, asset_server: Res<AssetServer>) {
    for tile in query.iter() {
        println!("Adding glTF: {}", tile.path);
        let _gltf_handle: Handle<Gltf> = asset_server.load(tile.path.as_str());
        let scene_handle = asset_server.get_handle(format!("{}#Scene0", tile.path).as_str());
        if let Some(trans) = &tile.transform {
            println!("TODO: Apply transformation {:?}", trans);
        }
        commands.spawn_scene(scene_handle);
    }
}

struct PntsTile {
    path: String,
    transform: Option<Vec<f32>>,
}

fn setup_pnts(
    mut commands: Commands,
    query: Query<&PntsTile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for tile in query.iter() {
        println!("Adding point tile mesh: {}", tile.path);
        let file = File::open(tile.path.as_str()).unwrap();
        let mut reader = BufReader::new(file);
        let pnts = Pnts::from_reader(&mut reader).unwrap();
        // dbg!(&pnts.feature_table.json);

        if let Some(dataref) = pnts.feature_table.json.position {
            assert_eq!(dataref.byte_offset, 0);
        }
        let points_length = pnts.feature_table.json.points_length as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(points_length);
        for _ in 0..points_length {
            positions.push([
                reader.read_f32::<LittleEndian>().unwrap(),
                reader.read_f32::<LittleEndian>().unwrap(),
                reader.read_f32::<LittleEndian>().unwrap(),
            ]);
        }
        if let Some(dataref) = pnts.feature_table.json.normal {
            println!("TODO: Read normals beginning at {}", dataref.byte_offset)
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

        let _batch_table = BatchTable::from_reader(
            &mut reader,
            pnts.header.batch_table_json_byte_length,
            pnts.header.batch_table_binary_byte_length,
        )
        .unwrap();
        // dbg!(&batch_table.json);

        if pnts.feature_table.json.rtc_center.is_some() {
            println!(
                "TODO: add transformation for rtc_center {:?}",
                pnts.feature_table.json.rtc_center
            );
        }
        let transform = if let Some(t) = &tile.transform {
            Transform::from_matrix(Mat4::from_cols_array(&[
                t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], t[8], t[9], t[10], t[11], t[12],
                t[13], t[14], t[15],
            ]))
        } else {
            Transform::identity()
        };
        println!("PntsTile transformation: {:?}", &transform);
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform,
            ..Default::default()
        });
    }
}

pub struct BoundingVolumeBox(Vec<f32>);

fn setup_bounding_volume(
    mut commands: Commands,
    bounding_volume_box: Res<BoundingVolumeBox>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // bounding_volume_box:
    // The first three elements define the x, y, and z values for the center of the box.
    // The next three elements (with indices 3, 4, and 5) define the x axis direction and half-length.
    // The next three elements (indices 6, 7, and 8) define the y axis direction and half-length.
    // The last three elements (indices 9, 10, and 11) define the z axis direction and half-length.
    let bvb = &bounding_volume_box.0;
    let vx = Vec3::new(bvb[3], bvb[4], bvb[5]);
    let vy = Vec3::new(bvb[6], bvb[7], bvb[8]);
    let vz = Vec3::new(bvb[9], bvb[10], bvb[11]);

    // LineStrip:
    // - Vertex data is a strip of lines. Each set of two adjacent vertices form a line.
    // - Vertices 0 1 2 3 create three lines 0 1, 1 2, and 2 3.
    let line_strips = vec![
        vec![
            vx + vy + vz,
            -vx + vy + vz,
            -vx + vy - vz,
            vx + vy - vz,
            vx + vy + vz,
        ],
        vec![
            vx - vy + vz,
            -vx - vy + vz,
            -vx - vy - vz,
            vx - vy - vz,
            vx - vy + vz,
        ],
        vec![vx + vy + vz, vx - vy + vz],
        vec![-vx + vy + vz, -vx - vy + vz],
        vec![-vx + vy - vz, -vx - vy - vz],
        vec![vx + vy - vz, vx - vy - vz],
    ];
    let material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    for line_strip in line_strips {
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        let vertices: Vec<[f32; 3]> = line_strip.into_iter().map(Into::into).collect();
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![0.0; vertices.len()]);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: material.clone(),
            transform: Transform::from_xyz(bvb[0], bvb[1], bvb[2]),
            ..Default::default()
        });
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(-2.0, 5.0, 50.0), // dragon
        Vec3::new(0., 0., 0.),
        // points transformed:
        // Vec3::new(1215031.0, -4736383.5, 4081666.3+50.0),
        // Vec3::new(1215031.0, -4736383.5, 4081666.3),
    ));
    // rotating light
    commands
        .spawn_bundle(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
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
