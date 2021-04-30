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
}

#[derive(FromArgs, PartialEq, Debug)]
/// Display GLTF example.
#[argh(subcommand, name = "display")]
struct Display {}

fn main() {
    let app: App = argh::from_env();
    match app.command {
        Commands::Display(_) => display(),
    }
}
