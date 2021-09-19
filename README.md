3D tiles
========

Rust library for reading and (eventually) writing [3D tiles](https://github.com/CesiumGS/3d-tiles/tree/main/specification).

Status:
- [x] Data structures with read and write (JSON only) support
- [ ] File reading API
- [ ] HTTP reading API

Contains an experimental viewer using the [Bevy](https://bevyengine.org/) game engine.


## Usage examples

View batched 3D model tileset:

    cargo run -- view data/3d-tiles-samples/TilesetWithDiscreteLOD/tileset.json

View point cloud tile:

    cargo run -- view data/3d-tiles-samples/TilesetWithExpiration/points.pnts

Extract glTF from batched 3D model tile:

    cargo run -- extract data/3d-tiles-samples/TilesetWithDiscreteLOD/dragon_medium.b3dm

View glTF scene file:

    cargo run -- view 3d-tiles-samples/TilesetWithDiscreteLOD/dragon_medium.glb

Extract glTF from instanced 3D model tile:

    cargo run -- extract data/3d-tiles-samples/TilesetWithTreeBillboards/tree_billboard.i3dm
