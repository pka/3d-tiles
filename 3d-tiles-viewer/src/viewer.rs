use crate::asset_loader::{Tiles3dAsset, Tiles3dAssetLoader};
use bevy::gltf::Gltf;
use bevy::log::{debug, warn};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::{pbr::AmbientLight, prelude::*};
use bevy_inspector_egui::{Inspectable, InspectableRegistry, WorldInspectorPlugin};
use bevy_prototype_debug_lines::*;
use byteorder::{LittleEndian, ReadBytesExt};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransform, LookTransformPlugin,
};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use tiles3d::b3dm::B3dm;
use tiles3d::batch_table::BatchTable;
use tiles3d::i3dm::I3dm;
use tiles3d::pnts::Pnts;
use tiles3d::tileset::{BoundingVolume, Tile, Tileset};

pub fn view_tileset(tileset_path: &str) {
    let mut app = App::build();
    init_viewer(&mut app);
    view_tileset_content(&mut app, tileset_path);
    app.run();
}

fn read_tileset_json(tileset_path: &str) -> Tileset {
    let file = File::open(tileset_path).expect(&format!("Couldn't open file {}", tileset_path));
    let tileset = Tileset::from_reader(BufReader::new(file)).expect("Invalid Tileset JSON");
    debug!("{:?}", &tileset);
    tileset
}

fn view_tileset_content(app: &mut AppBuilder, tileset_path: &str) {
    let tileset = read_tileset_json(tileset_path);
    let mut tile = &tileset.root;
    if tile.content.is_some() {
        view_tile(app, tileset_path, &tile, &tileset.root.bounding_volume);
    }
    while tile.content.is_none() {
        if let Some(ref children) = tile.children {
            for child in children {
                tile = child;
                if tile.content.is_some() {
                    view_tile(app, tileset_path, &tile, &tileset.root.bounding_volume);
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

fn view_tile(app: &mut AppBuilder, tileset_path: &str, tile: &Tile, root_volume: &BoundingVolume) {
    let tile_uri = &tile.content.as_ref().expect("Tile content missing").uri;
    let tile_fn = tile_fn(tileset_path, &tile_uri);
    debug!("view_tile {}", &tile_fn);
    let file = File::open(&tile_fn).expect(&format!("Couldn't open file {}", &tile_fn));
    let mut reader = BufReader::new(file);

    let transform = transform(&tile.transform);
    if let Some(ref bounding_volume_box) = root_volume.bounding_volume_box {
        app.world_mut().spawn().insert(BoundingVolumeBox {
            elements: bounding_volume_box.iter().map(|n| *n as f32).collect(),
            transform: transform.clone(),
        });
    }
    match Path::new(&tile_uri).extension().and_then(OsStr::to_str) {
        Some("b3dm") => {
            let b3dm = B3dm::from_reader(&mut reader).expect("Invalid b3dm");
            // debug!("{:?}", &b3dm.feature_table.header);
            // debug!("{:?}", &b3dm.batch_table.header);
            if b3dm.feature_table.header.rtc_center.is_some() {
                warn!(
                    "TODO: add transformation for rtc_center {:?}",
                    b3dm.feature_table.header.rtc_center
                );
            }
            view_gltf_from_reader(app, transform, &mut reader);
        }
        Some("i3dm") => {
            let i3dm = I3dm::from_reader(&mut reader).expect("Invalid i3dm");
            // debug!("{:?}", &i3dm.feature_table.header);
            // debug!("{:?}", &i3dm.batch_table.header);
            if i3dm.feature_table.header.rtc_center.is_some() {
                warn!(
                    "TODO: add transformation for rtc_center {:?}",
                    i3dm.feature_table.header.rtc_center
                );
            }

            if i3dm.header.gltf_format == 0 {
                let mut url = String::new();
                reader.read_to_string(&mut url).unwrap();
                debug!("{:?}", &url); // TODO
            } else if i3dm.header.gltf_format == 1 {
                view_gltf_from_reader(app, transform, &mut reader);
            }
        }
        Some("pnts") => {
            view_pnts(app, transform, &tile_fn);
        }
        Some("json") => {
            view_tileset_content(app, &tile_fn);
        }
        _ => {
            error!("Unknown file extension");
        }
    }
}

fn view_gltf_from_reader<R: Read>(app: &mut AppBuilder, transform: Transform, mut reader: R) {
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
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(DebugLinesPlugin)
        .insert_resource(
            InspectableRegistry::default()
                .with::<GltfTileComponent>()
                .with::<PntsTileComponent>()
                .with::<BoundingVolumeBox>(),
        )
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(setup_bounding_volume.system())
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

    // debugging
    app.add_system(light_debug_system.system());
    app.add_system(camera_debug_system.system());
}

/// Convert 3D tiles transform matrix to Bevy Transform
pub fn transform(transform: &Option<Vec<f32>>) -> Transform {
    if let Some(t) = transform {
        let mut t = Transform::from_matrix(Mat4::from_cols_array(&[
            t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7], t[8], t[9], t[10], t[11], t[12], t[13],
            t[14], t[15],
        ]));
        if t.scale != Vec3::ONE {
            warn!("Ignoring tile scale");
            t.scale = Vec3::ONE;
        }
        t
    } else {
        Transform::identity()
    }
}

pub fn view_gltf(app: &mut AppBuilder, transform: Transform, tile_path: &str) {
    app.world_mut().spawn().insert(GltfTileComponent {
        path: tile_path.to_owned(),
        transform,
    });
}

pub fn view_pnts(app: &mut AppBuilder, transform: Transform, tile_path: &str) {
    app.world_mut().spawn().insert(PntsTileComponent {
        path: tile_path.to_owned(),
        transform,
    });
}

#[derive(Inspectable)]
struct GltfTileComponent {
    path: String,
    transform: Transform,
}

fn setup_gltf(
    mut commands: Commands,
    query: Query<&GltfTileComponent>,
    asset_server: Res<AssetServer>,
) {
    // https://github.com/CesiumGS/3d-tiles/tree/1.0/specification#gltf-transforms
    let gltf_transform = Transform::from_matrix(Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]));
    for tile in query.iter() {
        debug!("Adding glTF: {}", tile.path);
        let _gltf_handle: Handle<Gltf> = asset_server.load(tile.path.as_str());
        let scene_handle = asset_server.get_handle(format!("{}#Scene0", tile.path).as_str());
        let transform = if tile.transform != Transform::identity() {
            tile.transform * gltf_transform
        } else {
            Transform::identity()
        };
        commands
            .spawn_bundle((transform, GlobalTransform::identity()))
            .with_children(|parent| {
                parent.spawn_scene(scene_handle);
            });
    }
}

#[derive(Inspectable)]
struct PntsTileComponent {
    path: String,
    transform: Transform,
}

fn setup_pnts(
    mut commands: Commands,
    query: Query<&PntsTileComponent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for tile in query.iter() {
        debug!("Adding point tile mesh: {}", tile.path);
        let file = File::open(tile.path.as_str()).unwrap();
        let mut reader = BufReader::new(file);
        let pnts = Pnts::from_reader(&mut reader).unwrap();
        // debug!("{:?}", &pnts.feature_table.header);

        if let Some(dataref) = pnts.feature_table.header.position {
            assert_eq!(dataref.byte_offset, 0);
        }
        let points_length = pnts.feature_table.header.points_length as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(points_length);
        for _ in 0..points_length {
            positions.push([
                reader.read_f32::<LittleEndian>().unwrap(),
                reader.read_f32::<LittleEndian>().unwrap(),
                reader.read_f32::<LittleEndian>().unwrap(),
            ]);
        }
        if let Some(dataref) = pnts.feature_table.header.normal {
            warn!("TODO: Read normals beginning at {}", dataref.byte_offset)
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
        // debug!("{:?}", &batch_table.header);

        if pnts.feature_table.header.rtc_center.is_some() {
            warn!(
                "TODO: add transformation for rtc_center {:?}",
                pnts.feature_table.header.rtc_center
            );
        }
        debug!("PntsTileComponent transformation: {:?}", &tile.transform);
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: tile.transform,
            ..Default::default()
        });
    }
}

#[derive(Inspectable)]
pub struct BoundingVolumeBox {
    elements: Vec<f32>,
    transform: Transform,
}

fn from_bounding_volume_box(elements: &Vec<f32>) -> (Vec3, Vec3, Vec3, Vec3) {
    // bounding_volume_box:
    // The first three elements define the x, y, and z values for the center of the box.
    // The next three elements (with indices 3, 4, and 5) define the x axis direction and half-length.
    // The next three elements (indices 6, 7, and 8) define the y axis direction and half-length.
    // The last three elements (indices 9, 10, and 11) define the z axis direction and half-length.
    let bvb = elements;
    /* 3D Tiles uses a right-handed Cartesian coordinate system.
       It defines the z axis as up for local Cartesian coordinate systems.
         Z
         |
     X---o
        /
       Y
    */
    // let center = Vec3::new(bvb[0], bvb[1], bvb[2]);
    // let vx = Vec3::new(bvb[3], bvb[4], bvb[5]);
    // let vy = Vec3::new(bvb[6], bvb[7], bvb[8]);
    // let vz = Vec3::new(bvb[9], bvb[10], bvb[11]);
    // Bevy/WebGPU coord system:
    let center = Vec3::new(-bvb[0], bvb[2], bvb[1]);
    let vx = -Vec3::new(-bvb[3], bvb[5], bvb[4]);
    let vz = Vec3::new(-bvb[6], bvb[8], bvb[7]);
    let vy = Vec3::new(-bvb[9], bvb[11], bvb[10]);
    (center, vx, vy, vz)
}

fn setup_bounding_volume(query: Query<&BoundingVolumeBox>, mut lines: ResMut<DebugLines>) {
    for bounding_volume_box in query.iter() {
        /* Bevy/WebGPU (from https://github.com/aevyrie/bevy_mod_bounding/blob/master/src/debug.rs)
              (2)-----(3)               Y
               | \     | \              |
               |  (1)-----(0) MAX       o---X
               |   |   |   |             \
          MIN (6)--|--(7)  |              Z
                 \ |     \ |
                  (5)-----(4)
        */

        let (center, vx, vy, vz) = from_bounding_volume_box(&bounding_volume_box.elements);
        let vertices = vec![
            vx + vy + vz,
            -vx + vy + vz,
            -vx + vy - vz,
            vx + vy - vz,
            vx - vy + vz,
            -vx - vy + vz,
            -vx - vy - vz,
            vx - vy - vz,
        ];

        let edges = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        let transform = Transform::from_translation(center) * bounding_volume_box.transform;
        for (p0, p1) in edges {
            lines.user_lines.push(Line::new(
                vertices[p0] + transform.translation,
                vertices[p1] + transform.translation,
                0.0,
                Color::GRAY,
                Color::GRAY,
            ));
        }
    }
}

fn setup_camera(mut commands: Commands, query: Query<&BoundingVolumeBox>) {
    if let Some(bounding_volume_box) = query.iter().next() {
        let (mut center, vx, vy, vz) = from_bounding_volume_box(&bounding_volume_box.elements);
        center += bounding_volume_box.transform.translation;
        let vs = bounding_volume_box.transform.scale;
        let (sx, sy, sz) = (vs[0], vs[1], vs[2]);
        // Vector from center to box corner (scaled with transform.scale)
        let v = vx * sx + vy * sy + vz * sz;
        let radius = v.length();

        let mut cam = PerspectiveCameraBundle::default();
        cam.perspective_projection.far = cam.perspective_projection.near + 20.0 * radius;
        commands.spawn_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            cam,
            // eye
            center + 3.0 * v,
            // target
            center,
        ));
        // rotating light
        let light = LightBundle {
            transform: Transform::from_translation(center + 2.0 * v),
            // light: Light { range: 4.0 * radius, ..Default::default()},
            ..Default::default()
        };
        commands.spawn_bundle(light); //.insert(Rotates);
    } else {
        commands.spawn_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            PerspectiveCameraBundle::default(),
            // Vec3::new(-2.0, 5.0, 50.0), // dragon
            // Vec3::new(0., 0., 0.),
            // points transformed:
            Vec3::new(1215031.0, -4736383.5, 4081666.3 + 50.0),
            Vec3::new(1215031.0, -4736383.5, 4081666.3),
        ));
        // rotating light
        commands
            .spawn_bundle(LightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..Default::default()
            })
            .insert(Rotates);
    }
}

/// this component indicates what entities should rotate
struct Rotates;

fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    // TODO: Handle rotation around translated center
    for mut transform in query.iter_mut() {
        *transform = Transform::from_rotation(Quat::from_rotation_y(
            (4.0 * std::f32::consts::PI / 20.0) * time.delta_seconds(),
        )) * *transform;
    }
}

fn light_debug_system(
    mut commands: Commands,
    query: Query<Entity, Changed<Light>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 100.0,
                    subdivisions: 5,
                })),
                material: materials.add(Color::rgb(1.0, 1.0, 0.0).into()),
                ..Default::default()
            });
        });
    }
}

fn camera_debug_system(
    cameras: Query<(&OrbitCameraController, &LookTransform, &Transform)>,
    mut lines: ResMut<DebugLines>,
) {
    for (_controller, transform, _scene_transform) in cameras.iter() {
        lines.line_gradient(
            transform.eye,
            transform.target,
            0.0,
            Color::RED,
            Color::YELLOW,
        );
    }
}
