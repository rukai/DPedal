use crate::output::OutDir;
use std::process::Command;

pub fn generate_config_app_wasm(dest_assets: OutDir) {
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
        dest_assets.create_compressed_file("dpedal_config_web_app_bg.wasm", &contents);

        let contents =
            std::fs::read("dpedal_config_web_app/target/generated/dpedal_config_web_app.js")
                .unwrap();
        dest_assets.create_compressed_file("dpedal_config_web_app.js", &contents);
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
