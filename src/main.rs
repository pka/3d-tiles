mod b3dm;
mod display;

use argh::FromArgs;
use display::display;

#[derive(FromArgs)]
/// 3D tiles reader.
struct App {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    Display(Display),
    Extract(Extract),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Display GLTF example.
#[argh(subcommand, name = "display")]
struct Display {}

#[derive(FromArgs, PartialEq, Debug)]
/// Extract GLTF binaries.
#[argh(subcommand, name = "extract")]
struct Extract {}

fn main() {
    let app: App = argh::from_env();
    match app.command {
        Commands::Display(_) => display(),
        Commands::Extract(_) => {
            b3dm::extract("assets/3d-tiles-samples/TilesetWithDiscreteLOD/dragon_low.b3dm").unwrap()
        }
    }
}
