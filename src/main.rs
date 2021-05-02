mod b3dm;
mod batch_table;
mod display;
mod error;
mod feature_table;
mod i3dm;
mod tileset;

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

#[derive(FromArgs, PartialEq, Default, Debug)]
/// Extract GLTF binaries.
#[argh(subcommand, name = "extract")]
struct Extract {
    #[argh(positional)]
    /// input file
    path: String,
}

fn main() {
    let app: App = argh::from_env();
    match app.command {
        Commands::Display(_) => display(),
        Commands::Extract(args) => {
            // b3dm::extract_glb(&args.path).unwrap();
            i3dm::extract_gltf(&args.path).unwrap();
        }
    }
}
