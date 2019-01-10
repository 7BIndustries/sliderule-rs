extern crate os_info;

use std::path::Path;
use std::process::Command;

/*
* Attempts to use npm, if installed, otherwise tries to mimic what npm would do.
*/
pub fn npm_install(target_dir: &Path, url: &str) -> super::SROutput {
    let mut output = super::SROutput { status: 0, wrapped_status: 0, stdout: Vec::new(), stderr: Vec::new() };
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

    // Try to run the npm command line and gather the output and errors so that they can be used later
    let stdoutput = match Command::new(&cmd_name).args(&vec).current_dir(target_dir).output() {
        Ok(out) => {
            out
        },
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                output.status = 200;
                output.stderr.push(String::from("ERROR: `npm` was not found, please install it."));
                return output;
            } else {
                output.status = 201;
                output.stderr.push(format!("ERROR: Could not install component from remote repository: {}", e));
                return output;
            }
        }
    };

    // If we don't get any errors, assume that the component was installed successfully
    if stdoutput.stderr.is_empty() {
        if !url.is_empty() {
            output.stdout.push(String::from("Component installed from remote repository."));
        }
        else {
            output.stdout.push(String::from("Sliderule project updated."));
        }
    }

    // Collect all of the other stdout entries
    if !stdoutput.stdout.is_empty() {
        output.stdout.push(String::from_utf8_lossy(&stdoutput.stdout).to_string());
    }

    // If there were errors, make sure we collect them
    if !stdoutput.stderr.is_empty() {
        output.stderr.push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
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