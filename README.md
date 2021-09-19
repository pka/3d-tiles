3D tiles
========

Rust library for reading and (eventually) writing [3D tiles](https://github.com/CesiumGS/3d-tiles/tree/main/specification).

Status:
- [x] Data structures with read and write (JSON only) support
- [ ] File reading API
- [ ] HTTP reading API

Contains an experimental viewer using the [Bevy](https://bevyengine.org/) game engine.


## Usage examples

View point cloud tile:

    cargo run -- display assets/3d-tiles-samples/TilesetWithExpiration/points.pnts

View glTF model:

    cargo run -- display 3d-tiles-samples/TilesetWithTreeBillboards/tree.glb
