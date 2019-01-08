extern crate os_info;

use std::path::Path;
use std::process::Command;

/*
* Attempts to use npm, if installed, otherwise tries to mimic what npm would do.
*/
pub fn npm_install(target_dir: &Path, url: &str) {
    let mut vec = Vec::new();
    vec.push("install");
    
    let info = os_info::get();
    let mut cmd_name = "npm";

    // Set the command name properly based on which OS the user is running
    if info.os_type() == os_info::Type::Windows {
        cmd_name = r"C:\Program Files\nodejs\npm.cmd";
    }

    // If no URL was specified, just npm update the whole project
    if !url.is_empty() {
        vec.push("--save");
        vec.push(url);
    }

    println!("Working...");

    let output = match Command::new(&cmd_name).args(&vec).current_dir(target_dir).output() {
        Ok(out) => {
            if !url.is_empty() {
                println!("Component installed from remote repository.");
            }
            else {
                println!("Sliderule project updated.");
            }

            out
        },
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                panic!("ERROR: `npm` was not found, please install it.");
            } else {
                panic!("ERROR: Could not install component from remote repository: {}", e);
            }
        }
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }
}


/*
* Uses the npm command to remove a remote component.
*/
pub fn npm_uninstall(target_dir: &Path, name: &str) {
    let mut vec = Vec::new();
    vec.push("uninstall");
    
    let info = os_info::get();
    let mut cmd_name = "npm";

    // Set the command name properly based on which OS the user is running
    if info.os_type() == os_info::Type::Windows {
        cmd_name = r"C:\Program Files\nodejs\npm.cmd";
    }

    // If no URL was specified, just npm update the whole project
    if !name.is_empty() {
        vec.push("--save");
        vec.push(name);
    }

    println!("Working...");

    // Attempt to install the component using npm
    match Command::new(&cmd_name).args(&vec).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Component uninstalled using npm.");
            out
        },
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                panic!("ERROR: `npm` was not found, please install it.");
            } else {
                panic!("ERROR: Could not install component from remote repository: {}", e);
            }
        }
    };

    // if !output.stderr.is_empty() {
    //     eprintln!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    // }
}