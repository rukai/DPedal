use askama::Template;
use clap::Parser;
use cli::Args;
use std::{path::Path, process::Command};

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
        dest_assets.create_file(
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

    generate_config_app_wasm(dest_assets);

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

fn generate_config_app_wasm(dest_assets: OutDir) {
    // build wasm app
    {
        let all_args = if env!("PROFILE") == "release" {
            vec!["build", "--release"]
        } else {
            vec!["build"]
        };
        run_command("dpedal_config_web_app", "cargo", &all_args);
    }

    // bindgen
    {
        let wasm_path = format!(
            "dpedal_config_web_app/target/wasm32-unknown-unknown/{}/dpedal_config_web_app.wasm",
            env!("PROFILE")
        );
        let destination_dir = "dpedal_config_web_app/target/generated";
        let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
        bindgen
            .web(true)
            .unwrap()
            .omit_default_module_path(false)
            .input_path(&wasm_path)
            .generate(destination_dir)
            .unwrap();

        // TODO: wasm-opt?
    }

    // compress
    {
        let contents =
            std::fs::read("dpedal_config_web_app/target/generated/dpedal_config_web_app_bg.wasm")
                .unwrap();
        dest_assets.create_file("dpedal_config_web_app_bg.wasm", &contents);

        let contents =
            std::fs::read("dpedal_config_web_app/target/generated/dpedal_config_web_app.js")
                .unwrap();
        dest_assets.create_file("dpedal_config_web_app.js", &contents);
    }
}

fn run_command(dir: &str, command: &str, args: &[&str]) -> String {
    let output = Command::new(command)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run the command {command} {args:?}\n{e}"));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if !output.status.success() {
        panic!("command {command} {args:?} failed:\nstdout:\n{stdout}\nstderr:\n{stderr}")
    }
    stdout
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
