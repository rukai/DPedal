use askama::Template;
use clap::Parser;
use cli::Args;
use std::path::Path;

use crate::output::OutDir;

mod cli;
mod output;
mod serve;

fn main() {
    // Set standard path to root of repo so this always runs in the same directory, regardless of where the user ran it from.
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(current_dir).unwrap();

    let args = Args::parse();

    let root = current_dir.join("site").join("root");
    std::fs::remove_dir_all(&root).ok();
    let out = OutDir::new(root);
    out.create_compressed_file("index.html", Landing {}.render().unwrap().as_bytes());

    // copy assets
    let dest_assets = out.join("assets");
    // TODO: properly include images in repo?
    for file in std::fs::read_dir("site/assets")
        .unwrap()
        .chain(std::fs::read_dir("../site_images").expect("Missing external site_images folder"))
    {
        let file = file.unwrap();
        dest_assets.create_compressed_file(
            file.file_name().to_str().unwrap(),
            &std::fs::read(file.path()).unwrap(),
        );
    }
    // browsers expect to find the file here.
    std::fs::rename("site/root/assets/favicon.ico", "site/root/favicon.ico").unwrap();

    // generate pages
    out.create_compressed_file("index.html", Landing {}.render().unwrap().as_bytes());
    out.create_compressed_file("build.html", Build {}.render().unwrap().as_bytes());
    out.create_compressed_file("config.html", Config {}.render().unwrap().as_bytes());

    if args.serve {
        println!("Hosting website at: http://localhost:8000");
        serve::serve(&current_dir.join("site").join("root"));
    } else {
        let out = current_dir.join("site").join("root");
        println!(
            "Succesfully generated website at: file://{}",
            out.to_str().unwrap()
        );
    }
}

#[derive(Template)]
#[template(path = "landing.html")]
struct Landing {}

#[derive(Template)]
#[template(path = "build.html")]
struct Build {}

#[derive(Template)]
#[template(path = "config.html")]
struct Config {}
