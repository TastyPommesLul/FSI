use inquire::{Confirm, Select, Text};
use std::fs::File;
use std::io::Write;
use terminal_spinners::SpinnerBuilder;

fn main() {
    let versions: Vec<&str> = vec![
        "26.2", "26.1.2", "26.1.1", "26.1", "1.21.11", "1.21.10", "1.21.9", "1.21.8", "1.21.7",
        "1.21.6", "1.21.5", "1.21.4", "1.21.3", "1.21.2", "1.21.1", "1.21",
    ];

    let gb_allocated_for_startup_script: Vec<&str> = vec![
        "1Gb", "2Gb", "3Gb", "4Gb", "5Gb", "6Gb", "7Gb", "8Gb", "9Gb", "10Gb", "12Gb", "14Gb",
        "16Gb", "20Gb", "24Gb", "32Gb",
    ];

    let seperate_folder =
        match Confirm::new("Do you want to download the server to a seperate folder?")
            .with_default(false)
            .with_help_message(
                "whether to download the server to a seperate folder or not (default: no)",
            )
            .prompt()
        {
            Ok(true) => true,
            Ok(false) => false,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(0);
            }
        };

    let mut path = String::new();

    if seperate_folder {
        match Text::new("Enter a folder name for the server")
            .with_help_message("the name of the folder to download the server to (default: server)")
            .with_default("server")
            .prompt()
        {
            Ok(name) => {
                path = name;
                match std::fs::create_dir(path.as_str()) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("failed to create dir at {}!{e}", path);
                        std::process::exit(1);
                    }
                };
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(0);
            }
        }
    }

    let version_choice = match Select::new("Select a version", versions)
        .with_help_message("the version of Fabric to download (default: latest)")
        .prompt()
    {
        Ok(ver) => ver,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    };

    let eula = match Confirm::new("Do you accept the EULA?")
        .with_help_message("whether to pre-accept the EULA or not (default: yes)")
        .with_default(true)
        .prompt()
    {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    };

    let file_name = match Text::new("Enter a file name for the server jar")
        .with_help_message("the name of the server jar file (default: server.jar)")
        .with_default("server.jar")
        .prompt()
    {
        Ok(name) => name,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    };

    let handle = SpinnerBuilder::new()
        .spinner(&terminal_spinners::DOTS)
        .text("Downloading Fabric Server...")
        .start();

    let full_path = format!("{path}/{file_name}");
    handle_download(
        version_choice,
        if seperate_folder {
            full_path.as_str()
        } else {
            file_name.as_str()
        },
    );

    let full_path = format!("{path}/eula.txt");
    match File::create(if seperate_folder {
        full_path.as_str()
    } else {
        "eula.txt"
    }) {
        Ok(mut file) => file.write_all(format!("eula={eula}\n").as_bytes()),
        Err(e) => {
            eprintln!(
                "failed to write file at {}! {e}",
                if seperate_folder {
                    full_path.as_str()
                } else {
                    "eula.txt"
                }
            );
            std::process::exit(1);
        }
    }
    .expect("Couldn't create eula.txt");

    handle.done();

    let os = std::env::consts::OS;

    let allocated_gb = match Select::new(
        "Select the amount of RAM allocated to the server",
        gb_allocated_for_startup_script,
    )
    .with_help_message("the amount of RAM allocated to the server (default: 1Gb)")
    .prompt()
    {
        Ok(gb) => gb,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    }
    .replace("b", "");

    let handle = SpinnerBuilder::new()
        .spinner(&terminal_spinners::DOTS)
        .text("Generating startup scripts...")
        .start();

    gen_linux(&path, &allocated_gb, &file_name);
    gen_windows(&path, &allocated_gb, &file_name);
    handle.done();

    let run_server = match Confirm::new("Do you want to run the server now?")
        .with_default(true)
        .with_help_message("whether to run the server now or not (default: yes)")
        .prompt()
    {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    };

    let os_ext = if os == "windows" { "bat" } else { "sh" };

    if run_server {
        if seperate_folder {
            std::env::set_current_dir(path).unwrap();
        }
        let mut command = std::process::Command::new(
            // Use OS-specific program for execution
            if os == "windows" {
                "cmd"
            } else if os == "linux" {
                "bash"
            } else {
                eprintln!("Usage of \"Run Now\", on a currently unsupported OS! {os}");
                std::process::exit(0)
            },
        );
        if os == "windows" {
            command.arg("/C").arg(format!("start.{os_ext}"));
        } else if os == "linux" {
            command.arg(format!("./start.{os_ext}"));
        }

        match command.spawn() {
            Ok(mut child) => match child.wait() {
                Ok(wait_status) => println!("Exited with status: {}", wait_status),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("child process failed to spawn! {e}");
                std::process::exit(1);
            }
        }
    }
}

fn gen_linux(path: &String, allocated_gb: &String, file_name: &String) {
    let full_path = if path.is_empty() {
        "start.sh".to_string()
    } else {
        format!("{path}/start.sh")
    };
    match File::create(full_path.clone()) {
        Ok(mut file) => file.write_all(
            format!("#!/usr/bin/env bash\njava -Xmx{allocated_gb} -jar {file_name} nogui")
                .as_bytes(),
        ),
        Err(e) => {
            eprintln!("failed to create file at {full_path}! {e}");
            std::process::exit(1);
        }
    }
    .expect("Couldn't create start.sh");
}
fn gen_windows(path: &String, allocated_gb: &String, file_name: &String) {
    let full_path = if path.is_empty() {
        "start.bat".to_string()
    } else {
        format!("{path}/start.bat")
    };
    match File::create(full_path.clone()) {
        Ok(mut file) => {
            file.write_all(format!("java -Xmx{allocated_gb} -jar {file_name} nogui").as_bytes())
        }
        Err(e) => {
            eprintln!("failed to create file at {full_path}! {e}");
            std::process::exit(1);
        }
    }
    .expect("Couldn't create start.bat");
}

fn handle_download(version: &str, file_path: &str) {
    let mut response = match reqwest::blocking::get(
        format!("https://meta.fabricmc.net/v2/versions/loader/{version}/0.19.3/1.1.2/server/jar")
            .as_str(),
    ) {
        Ok(res) => {
            if !res.status().is_success() {
                eprintln!(
                    "{}, Couldn't get server jar from fabric servers!",
                    res.status()
                );
                eprintln!(
                    "code={}, status={}",
                    res.status().as_u16(),
                    res.status().canonical_reason().unwrap_or("Unknown"),
                );
                std::process::exit(1);
            }
            res
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let mut file = match File::create(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("failed to create file! {e}");
            std::process::exit(1);
        }
    };

    match std::io::copy(&mut response, &mut file) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("failed to copy server jar from request! {e}");
            std::process::exit(1);
        }
    };
}
