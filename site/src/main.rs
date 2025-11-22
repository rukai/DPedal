use askama::Template;
use clap::Parser;
use cli::Args;
use std::path::Path;

mod cli;

fn main() {
    // Set standard path to root of repo so this always runs in the same directory, regardless of where the user ran it from.
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(current_dir).unwrap();

    let args = Args::parse();

    let root = current_dir.join("site").join("root");
    std::fs::remove_dir_all(&root).ok();
    std::fs::create_dir_all(&root).unwrap();

    // copy assets
    let dest_assets = root.join("assets");
    std::fs::create_dir_all(&dest_assets).unwrap();
    // TODO: properly include images in repo?
    for file in std::fs::read_dir("site/assets")
        .unwrap()
        .chain(std::fs::read_dir("../site_images").expect("Missing external site_images folder"))
    {
        let file = file.unwrap();
        std::fs::copy(file.path(), dest_assets.join(file.file_name())).unwrap();
    }
    // browsers expect to find the file here.
    std::fs::rename("site/root/assets/favicon.ico", "site/root/favicon.ico").unwrap();

    // generate pages
    std::fs::write(root.join("index.html"), Landing {}.render().unwrap()).unwrap();
    std::fs::write(root.join("build.html"), Build {}.render().unwrap()).unwrap();
    std::fs::write(root.join("config.html"), Config {}.render().unwrap()).unwrap();

    if args.serve {
        println!("Hosting website at: http://localhost:8000");

        devserver_lib::run(
            "localhost",
            8000,
            current_dir.join("site").join("root").to_str().unwrap(),
            false,
            "",
        );
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
