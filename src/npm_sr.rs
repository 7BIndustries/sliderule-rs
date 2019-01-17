extern crate os_info;

use std::path::Path;
use std::process::Command;

fn find_npm_windows() -> String {
    // Run the where command to attempt to find the npm.cmd script
    let output = match Command::new("where.exe").args(&["npm.cmd"]).output() {
        Ok(output) => output,
        Err(_) => {
            println!("Could not run where.exe which is needed for this CLI to work.");
            std::process::exit(2);
        }
    };

    let mut output_str = String::from("C:\\Program Files\\nodejs\\npm.cmd");

    // If there is not output, there will be no command path to extract
    if !output.stdout.is_empty() {
        // Convert the output into a string iterator that we can work with
        let lines = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = lines.split("\r\n").collect();

        // Take just the first line
        output_str = lines[0].trim().to_string();
    }

    output_str
}

/*
* Attempts to use npm, if installed, otherwise tries to mimic what npm would do.
*/
pub fn npm_install(target_dir: &Path, url: &str, cache: Option<String>) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    let mut vec = Vec::new();
    vec.push("install");

    let info = os_info::get();
    let mut cmd_name = String::from("npm");

    // Set the command name properly based on which OS the user is running
    if info.os_type() == os_info::Type::Windows {
        cmd_name = find_npm_windows(); //r"C:\Program Files\nodejs\npm.cmd";
    }

    // If the caller has selected to use a temporary cache, configure npm to use that
    if cache.is_some() {
        vec.push("--cache");
        vec.push(cache.as_ref().unwrap());
    }

    // If no URL was specified, just npm update the whole project
    if !url.is_empty() {
        vec.push("--save");
        vec.push(url);
    }

    // Try to run the npm command line and gather the output and errors so that they can be used later
    let stdoutput = match Command::new(&cmd_name)
        .args(&vec)
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                output.status = 200;
                output.stderr.push(String::from(
                    "ERROR: `npm` was not found, please install it.",
                ));
                return output;
            } else {
                output.status = 201;
                output.stderr.push(format!(
                    "ERROR: Could not install component from remote repository: {}",
                    e
                ));
                return output;
            }
        }
    };

    // If we don't get any errors, assume that the component was installed successfully
    if stdoutput.stderr.is_empty() {
        if !url.is_empty() {
            output
                .stdout
                .push(String::from("Component installed from remote repository."));
        } else {
            output.stdout.push(String::from(
                "Component successfully installed from remote repository.",
            ));
        }
    }

    // Collect all of the other stdout entrie
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    if !stdoutput.stderr.is_empty() {
        output
            .stderr
            .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
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
pub fn npm_uninstall(target_dir: &Path, name: &str, cache: Option<String>) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    let mut vec = Vec::new();
    vec.push("uninstall");

    let info = os_info::get();
    let mut cmd_name = String::from("npm");

    // Set the command name properly based on which OS the user is running
    if info.os_type() == os_info::Type::Windows {
        cmd_name = find_npm_windows();
    }

    // If the caller has selected to use a temporary cache, configure npm to use that
    if cache.is_some() {
        vec.push("--cache");
        vec.push(cache.as_ref().unwrap());
    }

    // If no URL was specified, just npm update the whole project
    if !name.is_empty() {
        vec.push("--save");
        vec.push(name);
    }

    // Attempt to install the component using npm
    let stdoutput = match Command::new(&cmd_name)
        .args(&vec)
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                output.status = 200;
                output.stderr.push(String::from(
                    "ERROR: `npm` was not found, please install it.",
                ));
                return output;
            } else {
                output.status = 202;
                output.stderr.push(format!(
                    "ERROR: Could not uninstall component from remote repository: {}",
                    e
                ));
                return output;
            }
        }
    };

    // If we don't get any errors, assume that the component was installed successfully
    if stdoutput.stderr.is_empty() {
        output.stdout.push(String::from(
            "Component successfully uninstalled from remote repository.",
        ));
    }

    // Collect all of the other stdout entrie
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    if !stdoutput.stderr.is_empty() {
        output
            .stderr
            .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
}
