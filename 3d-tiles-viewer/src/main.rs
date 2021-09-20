mod asset_loader;
mod viewer;

use argh::FromArgs;
use std::ffi::OsStr;
use std::path::Path;
use tiles3d::{b3dm, i3dm, pnts};
use viewer::{init_viewer, view_gltf, view_pnts, view_tileset};

#[derive(FromArgs)]
/// 3D tiles reader.
struct App {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    View(View),
    Extract(Extract),
}

#[derive(FromArgs, PartialEq, Debug)]
/// View 3D tile file.
#[argh(subcommand, name = "view")]
struct View {
    #[argh(positional)]
    /// input file
    path: String,
}

#[derive(FromArgs, PartialEq, Default, Debug)]
/// Extract GLTF binary from 3D tile.
#[argh(subcommand, name = "extract")]
struct Extract {
    #[argh(positional)]
    /// input file
    path: String,
}

fn main() {
    let app: App = argh::from_env();
    match app.command {
        Commands::View(args) => {
            if Path::new(&args.path).file_name().and_then(OsStr::to_str) == Some("tileset.json") {
                view_tileset(&args.path);
            } else {
                let mut app = bevy::app::App::build();
                init_viewer(&mut app);
                match Path::new(&args.path).extension().and_then(OsStr::to_str) {
                    Some("glb") => {
                        view_gltf(&mut app, None, &args.path);
                    }
                    Some("pnts") => {
                        view_pnts(&mut app, None, &args.path);
                    }
                    _ => {
                        println!("Unknown file extension");
                    }
                }
                app.run();
            }
        }
        Commands::Extract(args) => {
            match Path::new(&args.path).extension().and_then(OsStr::to_str) {
                Some("b3dm") => {
                    b3dm::extract_gltf(&args.path).unwrap();
                }
                Some("i3dm") => {
                    i3dm::extract_gltf(&args.path).unwrap();
                }
                Some("pnts") => {
                    pnts::extract(&args.path).unwrap();
                }
                _ => {
                    println!("Unknown file extension");
                }
            }
        }
    }
}
