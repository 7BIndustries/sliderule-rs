//! Sliderule is an implementation of the Distributed OSHW (Open Source Hardware) Framework ([`DOF`])
//! being developed by Mach 30 Foundation for Space Development.
//!
//! [`DOF`]: https://github.com/Mach30/dof
//!
//! Sliderule wraps the `git` and `npm` commands and uses them to manage DOF/Sliderule projects, both
//! on the local file system, and on a remote server. At this time only structure management is provided,
//! there is no capability to render models for documentation like assembly instructions out into their
//! distributable form.
//!
//! Central to understanding Sliderule is the concept of _local_ and _remote_ components. _Local_ components
//! are stored with a project, which is the top-level, enclosing component. Local components do not
//! have a repository associated with them, they are only stored in the `components` directory of a project.
//! Remote components, on the other hand, are stored in a remote repository, and are only installed into
//! the local file system if the user requests it. Remote components are intended to be shared, local components
//! are intended to only be used within their parent project. A local component can be converted into a remote
//! component later, if desired.
//!
//! The following is a list of the major operations available through this crate.
//! - _create_ - Creates a top level component unless creating a component inside of an existing component.
//!   In that case the new component is placed within the `components` directory of the parent "project" component.
//! - _add_ - Adds a remote component from a repository into the `node_modules` directory of the current component.
//! - _download_ - Downloads a copy of a component form a remote repository.
//! - _update_ - Downloads the latest changes to the component and/or its remote components (dependencies)
//! - _remove_ - Removes a component, whether it is local or remote.
//! - _upload_ - Uploads component changes on the local file system to its remote repository.
//! - _refactor_ - Converts a local component to a remote component so that it may be more easily shared.
//!
//! There are also various helper functions to do things like getting what level a component is in a hierarchy and
//! compiling the licenses of all components in a project.

#![allow(dead_code)]

extern crate liquid;
extern crate os_info;
extern crate walkdir;

use std::cmp::Ordering;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub struct SROutput {
    pub status: i32,
    pub wrapped_status: i32,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

/// Creates a new component or converts an existing directory into a component.
///
/// If `target_dir` is not a component directory, a new, top-level project component will be created.
/// If `target_dir` is a component directory, a new component is created in the existing `components`
/// directory. The name of the component is determine by the `name` parameter. Names are not allowed
/// to include dots. The source materials license `source_license` and documentation license (`doc_license`)
/// must be specified and must be from the [`SPDX`] license list.
///
/// [`SPDX`]: https://spdx.org/licenses/
///
/// # Examples
///
/// Creating a new top-level project component:
/// ```
/// extern crate sliderule;
///
/// let temp_dir = std::env::temp_dir();
///
/// let output = sliderule::create_component(
///     &temp_dir,
///     String::from("newproject"),
///     String::from("TestSourceLicense"),
///     String::from("TestDocLicense"),
/// );
///
/// assert!(temp_dir.join("newproject").exists());
/// ```
/// Creating a new local component:
/// ```
/// extern crate sliderule;
///
/// let temp_dir = std::env::temp_dir().join("newproject");
///
/// let output = sliderule::create_component(
///     &temp_dir,
///     String::from("localcomponent"),
///     String::from("TestSourceLicense"),
///     String::from("TestDocLicense"),
/// );
///
/// assert!(temp_dir.join("components").join("localcomponent").exists());
/// ```

pub fn create_component(
    target_dir: &Path,
    name: String,
    source_license: String,
    doc_license: String,
) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    // The path can either lead to a top level component (project), or a component nested within a project
    let component_dir: PathBuf;

    // This is a top level component (project)
    if target_dir.join(".sr").exists() {
        component_dir = target_dir.join("components").join(&name);
    } else {
        component_dir = target_dir.join(&name);
    }

    // Create a directory for our component
    match fs::create_dir(&component_dir) {
        Ok(_) => (),
        Err(e) => {
            output.status = 11;
            output.stderr.push(format!(
                "ERROR: Could not create component directory: {}",
                e
            ));
        }
    };

    // Create the components directory, if needed
    if !component_dir.join("components").exists() {
        match fs::create_dir(component_dir.join("components")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 12;
                output.stderr.push(format!(
                    "ERROR: Could not create components directory: {}",
                    e
                ));
            }
        };

        // Create a placeholder file to ensure that the directory gets pushed to the repo
        match fs::File::create(component_dir.join("components").join(".ph")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 21;
                output.stderr.push(format!(
                    "ERROR: Could not create placeholder file in components directory: {}",
                    e
                ));
            }
        };
    } else {
        output.stdout.push(String::from(
            "components directory already exists, using existing directory.",
        ));
    }

    // Create the dist directory, if needed
    if !component_dir.join("dist").exists() {
        match fs::create_dir(component_dir.join("dist")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 13;
                output
                    .stderr
                    .push(format!("ERROR: Could not create dist directory: {}", e));
            }
        };

        // Create a placeholder file to ensure that the directory gets pushed to the repo
        match fs::File::create(component_dir.join("dist").join(".ph")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 21;
                output.stderr.push(format!(
                    "ERROR: Could not create placeholder file in dist directory: {}",
                    e
                ));
            }
        };
    } else {
        output.stdout.push(String::from(
            "dist directory already exists, using existing directory.",
        ));
    }

    // Create the docs directory, if needed
    if !component_dir.join("docs").exists() {
        match fs::create_dir(component_dir.join("docs")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 14;
                output
                    .stderr
                    .push(format!("ERROR: Could not create docs directory: {}", e));
            }
        };

        // Create a placeholder file to ensure that the directory gets pushed to the repo
        match fs::File::create(component_dir.join("docs").join(".ph")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 21;
                output.stderr.push(format!(
                    "ERROR: Could not create placeholder file in docs directory: {}",
                    e
                ));
            }
        };
    } else {
        output.stdout.push(String::from(
            "docs directory already exists, using existing directory.",
        ));
    }

    //Create the source directory, if needed
    if !component_dir.join("source").exists() {
        match fs::create_dir(component_dir.join("source")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 15;
                output
                    .stderr
                    .push(format!("ERROR: Could not create source directory: {}", e));
            }
        };

        // Create a placeholder file to ensure that the directory gets pushed to the repo
        match fs::File::create(component_dir.join("source").join(".ph")) {
            Ok(_) => (),
            Err(e) => {
                output.status = 21;
                output.stderr.push(format!(
                    "ERROR: Could not create placeholder file in source directory: {}",
                    e
                ));
            }
        };
    } else {
        output.stdout.push(String::from(
            "source directory already exists, using existing directory.",
        ));
    }

    // Generate the template readme file
    let file_output = generate_readme(&component_dir, &name);
    output = combine_sroutputs(output, file_output);

    // Generate bom_data.yaml
    let file_output = generate_bom(&component_dir, &name);
    output = combine_sroutputs(output, file_output);

    // Generate package.json, if needed
    let file_output = generate_package_json(&component_dir, &name, &source_license);
    output = combine_sroutputs(output, file_output);

    // Generate the .sr file that provides extra information about this component
    let file_output = generate_dot_file(&component_dir, &source_license, &doc_license);
    output = combine_sroutputs(output, file_output);

    // Make sure that our package.json file is updated with all the license info
    let amal_output = amalgamate_licenses(&component_dir);
    output = combine_sroutputs(output, amal_output);

    output
        .stdout
        .push(String::from("Finished setting up component."));

    output
}

/// Allows a user to set the username and password for a component's remote URL.
/// This can be a security risk on multi-user systems since the password is stored in plain text inside
/// the .git/config file. Users should be encouraged to use ssh instead of https to avoid this security issue.
pub fn remote_login(
    target_dir: &Path,
    url: Option<String>,
    username: Option<String>,
    password: Option<String>,
) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    let mut final_url = url.unwrap().to_owned();
    if final_url.contains("https") {
        // Format the https string properly to contain the username and password
        final_url = add_user_pass_to_https(final_url, username, password);
    }

    // Initialize as a repo only if needed
    if !target_dir.join(".git").exists() {
        // Initialize the git repository and set the remote URL to push to
        let git_output = git_sr::git_init(target_dir, &final_url);
        output = combine_sroutputs(output, git_output);
    } else {
        // Change/set the remote URL of the component
        let git_output = git_sr::git_set_remote_url(target_dir, &final_url);
        output = combine_sroutputs(output, git_output);
    }

    output
}

/// Uploads any changes to the project/component to a remote repository.
///
/// The remote repository at `url` must exist before trying to upload changes to it.
/// `target_dir` must be a valid Sliderule component directory.
/// `messages` should describe the changes that were made since the last upload.
///
/// # Examples
///
/// ```no_run
/// let temp_dir = std::env::temp_dir();
///
/// let output = sliderule::upload_component(
///     &temp_dir.join("newproject"),
///     String::from("Initial commit"),
///     String::from("https://repo.com/user/newproject"),
///     None,
///     None
/// );
/// ```
pub fn upload_component(
    target_dir: &Path,
    message: String,
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> SROutput {
    // Make sure that our package.json file is updated with all the license info
    let mut output = amalgamate_licenses(&target_dir);

    // Initialize as a repo only if needed
    if !target_dir.join(".git").exists() {
        let mut final_url = url.to_owned();
        if final_url.contains("https") {
            // Format the https string properly to contain the username and password
            final_url = add_user_pass_to_https(final_url, username, password);
        }

        // Initialize the git repository and set the remote URL to push to
        let git_output = git_sr::git_init(target_dir, &final_url);
        output = combine_sroutputs(output, git_output);
    }

    // Create the gitignore file only if we need to
    if !target_dir.join(".gitignore").exists() {
        // Generate gitignore file so that we don't commit and push things we shouldn't be
        let file_output = generate_gitignore(&target_dir);
        output = combine_sroutputs(output, file_output);
    }

    // Add all changes, commit and push
    let git_output = git_sr::git_add_and_commit(target_dir, message);

    // Combine the outputs together
    output = combine_sroutputs(output, git_output);

    output
        .stdout
        .push(String::from("Done uploading component."));

    output
}

fn add_user_pass_to_https(
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> String {
    let mut userpass = String::new();
    let mut final_url = String::new();

    // If we have a username and password, rework the URL to store them
    if username.is_some() && password.is_some() {
        userpass.push_str("https://");
        userpass.push_str(&username.unwrap());
        userpass.push_str(":");
        userpass.push_str(&password.unwrap());
        userpass.push_str("@");

        final_url = url.replace("https://", &userpass);
    }

    final_url
}

/// Converts a local component into a remote component, uploading it to the remote repo and then
/// installing via npm.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `name` is the name of the component in the `components` directory to refactor.
/// `url` is the remote URL to push the component to. This URL must exist before this is called.
///
/// # Examples
///
/// ```no_run
/// let temp_dir = std::env::temp_dir();
///
/// let output = sliderule::refactor(
///     &temp_dir.join("newproject"),
///     String::from("level1_component"),
///     String::from("https://repo.com/user/level1_component"),
///     None,
///     None
/// );
/// ```
pub fn refactor(
    target_dir: &Path,
    name: String,
    url: String,
    username: Option<String>,
    password: Option<String>,
) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    let component_dir = target_dir.join("components").join(&name);

    let mut remote_url = String::new();
    if url.starts_with("git@") {
        remote_url.push_str("git+ssh://");
        remote_url.push_str(&url);
    } else {
        remote_url = url.to_owned();
    }

    if component_dir.exists() {
        // Upload the current component to the remote repo
        output = upload_component(
            &component_dir,
            String::from("Initial commit, refactoring component"),
            url.to_owned(),
            username,
            password,
        );

        // Remove the local component
        let remove_output = remove(&target_dir, &name);
        output = combine_sroutputs(output, remove_output);

        // Install the newly minted remote component using npm
        let add_output = add_remote_component(&target_dir, &remote_url, None);
        output = combine_sroutputs(output, add_output);

        // Shouldn't need it here, but make sure that our package.json file is updated with all the license info
        let amal_output = amalgamate_licenses(&target_dir);
        output = combine_sroutputs(output, amal_output);
    } else {
        output.status = 10;
        output.stderr.push(String::from(
            "ERROR: The component does not exist in the components directory.",
        ));
        return output;
    }

    output.stdout.push(String::from(
        "Finished refactoring local component to remote repository.",
    ));

    output
}

/// Removes a component (local or remote) from the project directory structure.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `name` must be a valid name for a component in either the `components` or
/// the `node_modules` directories.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// // Remove a local component so we can test it
/// let output = sliderule::remove(&test_dir.join("toplevel"), "level1");
///
/// // Make sure that the level1 directory was removed
/// assert!(!&test_dir
///         .join("toplevel")
///         .join("components")
///         .join("level1")
///         .exists());
/// ```
pub fn remove(target_dir: &Path, name: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    let component_dir = target_dir.join("components").join(name);

    // If the component exists as a subdirectory of components delete the directory directly otherwise use npm to remove it.
    if component_dir.exists() {
        output
            .stdout
            .push(format!("Deleting component directory {}.", name));

        // Step through every file and directory in the path to be deleted and make sure that none are read-only
        for entry in walkdir::WalkDir::new(&component_dir) {
            let entry = match entry {
                Ok(ent) => ent,
                Err(e) => {
                    output.status = 6;
                    output.stderr.push(format!(
                        "ERROR: Could not handle entry while walking components directory tree: {}",
                        e
                    ));
                    return output;
                }
            };

            // Remove read-only permissions on every entry
            let md = match entry.path().metadata() {
                Ok(m) => m,
                Err(e) => {
                    output.status = 7;
                    output.stderr.push(format!(
                        "ERROR: Could not get metadata for a .git directory entry: {}",
                        e
                    ));
                    return output;
                }
            };

            // Set the permissions on the directory to make sure that we can delete it when the time comes
            let mut perms = md.permissions();
            perms.set_readonly(false);
            match fs::set_permissions(&entry.path(), perms) {
                Ok(_) => (),
                Err(e) => {
                    output.status = 8;
                    output.stderr.push(format!(
                        "ERROR: Failed to set permissions on .git directory: {}",
                        e
                    ));
                    return output;
                }
            };
        }

        // Delete the directory recursively
        match fs::remove_dir_all(component_dir) {
            Ok(_) => (),
            Err(e) => {
                output.status = 9;
                output.stderr.push(format!(
                    "ERROR: not able to delete component directory: {}",
                    e
                ));
                return output;
            }
        };
    } else {
        output = remove_remote_component(&target_dir, name, None);
    }

    // Make sure that our package.json file is updated with all the license info
    let amal_output = amalgamate_licenses(&target_dir);

    // Roll the amalgamation output in with what we have already
    let mut output = combine_sroutputs(output, amal_output);

    // Let the caller know the component was removed successfully
    output
        .stdout
        .push(format!("Component {} was successfully removed.", name));

    output
}

/// Allows the user to change the source and/or documentation licenses for the project.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `source_license` Must be an SPDX compliant string for the component's source files (mechanical/electrical CAD, etc)
/// `doc_license` Must be an SPDX compliant string for the documentation content of the component
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let output = sliderule::change_licenses(
///    &test_dir.join("toplevel"),
///    String::from("TestSourceLicense"),
///    String::from("TestDocLicense"),
///    );
///
/// assert_eq!(0, output.status);
/// assert!(output.stderr.is_empty());
/// let content = fs::read_to_string(test_dir.join("toplevel")
///    .join(".sr"))
///    .expect("Unable to read file");
///
/// assert!(content.contains("TestSourceLicense"));
/// assert!(content.contains("TestDocLicense"));
/// ```
pub fn change_licenses(target_dir: &Path, source_license: String, doc_license: String) -> SROutput {
    // Update the source and documentation licenses
    let output = update_yaml_value(&target_dir.join(".sr"), "source_license", &source_license);
    let secondary_output = update_yaml_value(
        &target_dir.join(".sr"),
        "documentation_license",
        &doc_license,
    );

    // Combine the outputs from the attempts to change the source and documentation licenses
    let output = combine_sroutputs(output, secondary_output);

    // Make sure our new licenses are up to date in package.json
    let amal_output = amalgamate_licenses(&target_dir);

    // Combine the previously combined output with the new output from the license amalgamation
    let output = combine_sroutputs(output, amal_output);

    output
}

/*
 *
*/
/// Adds a component from the remote repository at the provided URL to the node_modules directory.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `url` URL of the repository the remote component resides in.
/// 'cache` Allows a user to specify a temporary cache for npm to use. Mostly for testing purposes.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
/// # let cache_dir = temp_dir.join(format!("cache_{}", uuid::Uuid::new_v4()));
///
/// let output = sliderule::add_remote_component(
///     &test_dir.join("toplevel"),
///     "https://github.com/jmwright/arduino-sr.git",
///     Some(cache_dir.to_string_lossy().to_string()),
/// );
///
/// assert_eq!(0, output.status);
///
/// let component_path = test_dir
///     .join("toplevel")
///     .join("node_modules")
///     .join("arduino-sr");
///
/// assert!(component_path.exists());
/// ```
pub fn add_remote_component(target_dir: &Path, url: &str, cache: Option<String>) -> SROutput {
    let mut output = npm_sr::npm_install(target_dir, &url, cache);

    // Make sure that our package.json file is updated with all the license info
    let amal_output = amalgamate_licenses(&target_dir);
    output = combine_sroutputs(output, amal_output);

    if output.status != 0 || output.wrapped_status != 0 {
        output.stderr.push(String::from(
            "ERROR: Remote component was not successfully added",
        ));
    }

    if output.status == 0 && output.wrapped_status == 0 {
        output
            .stdout
            .push(String::from("Remote component was added successfully."));
    }

    output
}

/// Removes a remote component via the name.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `name` name of the component to remove. The node_modules directory is assumed, so name conflicts
/// with local components are ignored.
/// 'cache` Allows a user to specify a temporary cache for npm to use. Mostly for testing purposes.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
/// # let cache_dir = temp_dir.join(format!("cache_{}", uuid::Uuid::new_v4()));
///
/// let output = sliderule::remove_remote_component(
///            &test_dir.join("toplevel"),
///            "blink_firmware",
///            Some(cache_dir.to_string_lossy().to_string()),
///        );
///
/// assert_eq!(0, output.status);
///
/// assert!(!test_dir
///     .join("toplevel")
///     .join("node_modules")
///     .join("blink_firmware")
///     .exists());
/// ```
pub fn remove_remote_component(target_dir: &Path, name: &str, cache: Option<String>) -> SROutput {
    // Use npm to remove the remote component
    let mut output = npm_sr::npm_uninstall(target_dir, name, cache);

    if output.status != 0 || output.wrapped_status != 0 {
        output.stderr.push(String::from(
            "ERROR: Component was not successfully removed",
        ));
    }

    if output.status == 0 && output.wrapped_status == 0 {
        output
            .stdout
            .push(String::from("Component was removed successfully."));
    }

    output
}

/// Downloads a copy of a component from the remote repository at the specified URL.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `url` URL of the remote repository to download the component from.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let output = sliderule::download_component(
///             &test_dir.join("toplevel"),
///             "https://github.com/jmwright/toplevel.git",
///         );
///
/// assert_eq!(0, output.status);
///
/// assert!(output.stdout[1].contains("Component was downloaded successfully."));
/// ```
pub fn download_component(target_dir: &Path, url: &str) -> SROutput {
    let mut output = git_sr::git_clone(target_dir, url);

    if output.status != 0 || output.wrapped_status != 0 {
        output.stderr.push(String::from(
            "ERROR: Component was not successfully downloaded",
        ));
    }

    if output.status == 0 && output.wrapped_status == 0 {
        output
            .stdout
            .push(String::from("Component was downloaded successfully."));
    }

    output
}

/// Updates all remote component in the node_modules directory.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let output = sliderule::update_dependencies(&test_dir.join("toplevel"));
///
/// assert_eq!(0, output.status);
///
/// assert!(output.stdout[1].contains("Dependencies were updated successfully."));
/// ```
pub fn update_dependencies(target_dir: &Path) -> SROutput {
    let mut output = npm_sr::npm_install(target_dir, "", None);

    if output.status != 0 || output.wrapped_status != 0 {
        output.stderr.push(String::from(
            "ERROR: Dependencies were not successfully updated",
        ));
    }

    if output.status == 0 && output.wrapped_status == 0 {
        output
            .stdout
            .push(String::from("Dependencies were updated successfully."));
    }

    // Make sure that our package.json file is updated with all the license info
    let amal_output = amalgamate_licenses(&target_dir);
    output = combine_sroutputs(output, amal_output);

    output
}

/*
 * Updates the local component who's directory we're in
*/
/// Downloads updates from the remote repository that is set for this directory.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let output = sliderule::update_local_component(&test_dir.join("toplevel"));
///
/// assert_eq!(0, output.status);
///
/// assert_eq!(output.stdout[0].trim(), "Already up to date.");
/// assert_eq!(output.stdout[1], "Component updated successfully.");
/// ```
pub fn update_local_component(target_dir: &Path) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if target_dir.join(".git").exists() {
        output = git_sr::git_pull(target_dir);

        // Make sure that our package.json file is updated with all the license info
        let amal_output = amalgamate_licenses(&target_dir);
        output = combine_sroutputs(output, amal_output);

        // Give the user an idea of whether the update was successful or not
        if output.status == 0 {
            output
                .stdout
                .push(String::from("Component updated successfully."));
        } else {
            output
                .stdout
                .push(String::from("Component not updated successfully."));
        }
    } else {
        output.status = 1;
        output.stderr.push(String::from(
            "ERROR: Component is not set up as a repository, cannot update it.",
        ));
    }

    output
}

/// Prints out each of the licenses in the component's directory tree so that
/// users can see what licenses are in use and where they reside.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let license_listing = sliderule::list_all_licenses(&test_dir.join("toplevel"));
///
/// assert!(license_listing.contains("Licenses Specified In This Component:"));
/// assert!(license_listing.contains("Unlicense"));
/// assert!(license_listing.contains("CC0-1.0"));
/// assert!(license_listing.contains("NotASourceLicense"));
/// assert!(license_listing.contains("NotADocLicense"));
/// assert!(license_listing.contains("CC-BY-4.0"));
/// ```
pub fn list_all_licenses(target_dir: &Path) -> String {
    let nl = get_newline();
    let mut license_listing = String::from("Licenses Specified In This Component:");
    license_listing.push_str(&nl);

    // Get the ordered listing of the component hierarchy
    let sr_entries = get_sr_paths(target_dir);

    // Compile the licenses of all the entries
    for entry in sr_entries {
        // We want the licenses from our current dot files
        let source_value = get_yaml_value(&entry, "source_license");
        let doc_value = get_yaml_value(&entry, "documentation_license");

        license_listing.push_str(&format!(
            "Path: {}, Source License: {}, Documentation License: {}{}",
            entry.display(),
            source_value,
            doc_value,
            nl
        ));
    }

    license_listing
}

/// Extracts the source and documentation licenses from a component's .sr file.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// # Examples
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let licenses = sliderule::get_licenses(&test_dir);
///
/// assert_eq!(licenses.0, "Unlicense");
/// assert_eq!(licenses.1, "CC0-1.0");
/// ```
pub fn get_licenses(target_dir: &Path) -> (String, String) {
    let sr_file: PathBuf;

    // We can hand back the default licenses, if nothing else
    let mut source_license = String::from("Unlicense");
    let mut doc_license = String::from("CC0-1.0");

    // If we're in a component directory, pull the license info from that
    sr_file = target_dir.join(".sr");

    // Safety check to make sure the file exists
    if sr_file.exists() {
        // Extract the licenses from the file
        source_license = get_yaml_value(&sr_file, "source_license");
        doc_license = get_yaml_value(&sr_file, "documentation_license");
    }

    (source_license, doc_license)
}

/// Figures out and returns what depth within another component's hierarchy
/// the component is at.
/// 0 = A top level component is probably being created
/// 1 = A top level component with no parent
/// 2 = A sub-component at depth n
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// # Examples
///
/// ```
/// # use std::fs;
/// # let temp_dir = std::env::temp_dir();
/// # let url = "https://github.com/jmwright/toplevel.git";
/// # let uuid_dir = uuid::Uuid::new_v4();
/// # let test_dir_name = format!("temp_{}", uuid_dir);
/// # fs::create_dir(temp_dir.join(&test_dir_name)).expect("Unable to create temporary directory.");
/// # match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join("toplevel")) {
/// # Ok(repo) => repo,
/// # Err(e) => panic!("failed to clone: {}", e),
/// # };
/// # let test_dir = temp_dir.join(test_dir_name);
///
/// let level = sliderule::get_level(&test_dir.join("components").join("level1"));
///
/// assert_eq!(0, level)
/// ```
pub fn get_level(target_dir: &Path) -> u8 {
    let level: u8;

    // Allows us to check if there is a .sr file in the current directory
    let current_file = target_dir.join(".sr");

    // Allows us to check if there is a .sr file in the parent directory
    let parent_file = target_dir.join(".sr");

    // If the parent directory contains a .sr file, we have a sub-component, if not we have a top level component
    if !parent_file.exists() && !current_file.exists() {
        level = 0;
    } else if !parent_file.exists() && current_file.exists() {
        level = 1;
    } else {
        level = 2;
    }

    level
}

/// Simply returns the version number of this crate.
/// May be expanded later to include a build number or sha checksum.
///
/// # Examples
///
/// ```
/// let version_num = sliderule::get_version();
///
/// assert_eq!(version_num, "0.2.1");
/// ```
pub fn get_version() -> String {
    let version = String::from("0.2.1");

    return version;
}

/*
 * Generates a template README.md file to help the user get started.
*/
fn generate_readme(target_dir: &Path, name: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if !target_dir.join("README.md").exists() {
        // Add the things that need to be put substituted into the README file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

        let contents = render_template("README.md.liquid", &mut globals);

        // Write the template text into the readme file
        match fs::write(target_dir.join("README.md"), contents) {
            Ok(_) => (),
            Err(e) => {
                output.status = 16;
                output
                    .stderr
                    .push(format!("Could not write to README.md file: {}", e));
            }
        };
    } else {
        output.stdout.push(String::from(
            "README.md already exists, using existing file and refusing to overwrite.",
        ));
    }

    output
}

/*
 * Generates a bill of materials from a template.
*/
fn generate_bom(target_dir: &Path, name: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if !target_dir.join("bom_data.yaml").exists() {
        // Add the things that need to be put substituted into the BoM file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

        let contents = render_template("bom_data.yaml.liquid", &mut globals);

        // Write the template text into the readme file
        match fs::write(target_dir.join("bom_data.yaml"), contents) {
            Ok(_) => (),
            Err(e) => {
                output.status = 17;
                output
                    .stderr
                    .push(format!("Could not write to bom_data.yaml: {}", e));
            }
        };
    } else {
        output.stdout.push(String::from(
            "bom_data.yaml already exists, using existing file and refusing to overwrite.",
        ));
    }

    output
}

/*
 * Generates a package.json file for npm based on a Liquid template.
*/
fn generate_package_json(target_dir: &Path, name: &str, license: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if !target_dir.join("package.json").exists() {
        // Add the things that need to be put substituted into the package file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));
        globals.insert(
            "license".into(),
            liquid::value::Value::scalar(license.to_owned()),
        );

        let contents = render_template("package.json.liquid", &mut globals);

        // Write the contents into the file
        match fs::write(target_dir.join("package.json"), contents) {
            Ok(_) => (),
            Err(e) => {
                output.status = 18;
                output
                    .stderr
                    .push(format!("Could not write to package.json: {}", e));
            }
        };
    } else {
        output.stdout.push(String::from(
            "package.json already exists, using existing file and refusing to overwrite.",
        ));
    }

    output
}

/*
 * Generates the .gitignore file used by the git command to ignore files and directories.
*/
fn generate_gitignore(target_dir: &Path) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if !target_dir.join(".gitignore").exists() {
        // Add the things that need to be put substituted into the gitignore file (none at this time)
        let mut globals = liquid::value::Object::new();

        let contents = render_template(".gitignore.liquid", &mut globals);

        // Write the contents to the file
        match fs::write(target_dir.join(".gitignore"), contents) {
            Ok(_) => (),
            Err(e) => {
                output.status = 19;
                output
                    .stderr
                    .push(format!("Could not write to .gitignore: {}", e));
            }
        };
    } else {
        output.stdout.push(String::from(
            ".gitignore already exists, using existing file and refusing to overwrite.",
        ));
    }

    output
}

/*
 * Generates the dot file that tracks whether this is a top level component/project or a sub-component
*/
fn generate_dot_file(target_dir: &Path, source_license: &str, doc_license: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stderr: Vec::new(),
        stdout: Vec::new(),
    };

    if !target_dir.join(".sr").exists() {
        // Add the things that need to be put substituted into the .top file (none at this time)
        let mut globals = liquid::value::Object::new();
        globals.insert(
            "source_license".into(),
            liquid::value::Value::scalar(source_license.to_owned()),
        );
        globals.insert(
            "doc_license".into(),
            liquid::value::Value::scalar(doc_license.to_owned()),
        );

        let contents = render_template(".sr.liquid", &mut globals);

        // Write the contents to the file
        match fs::write(target_dir.join(".sr"), contents) {
            Ok(_) => (),
            Err(e) => {
                output.status = 20;
                output
                    .stderr
                    .push(format!("Could not write to .sr file: {}", e));
            }
        };
    } else {
        output.stdout.push(String::from(
            ".sr already exists, using existing file and refusing to overwrite.",
        ));
    }

    output
}

/*
 * Reads a template to a string so that it can be written to a new components directory structure.
*/
fn render_template(template_name: &str, globals: &mut liquid::value::Object) -> String {
    let mut contents = String::new();

    if template_name == ".sr.liquid" {
        contents = templates::sr_file_template();
    } else if template_name == ".gitignore.liquid" {
        contents = templates::gitignore_template();
    } else if template_name == "bom_data.yaml.liquid" {
        contents = templates::bom_data_yaml_template();
    } else if template_name == "package.json.liquid" {
        contents = templates::package_json_template();
    } else if template_name == "README.md.liquid" {
        contents = templates::readme_template();
    }

    // Render the output of the template using Liquid
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .parse(&contents)
        .expect("Could not parse template using Liquid.");

    let output = template
        .render(globals)
        .expect("Could not render template using Liquid.");

    output
}

/*
 * Walk the directory structure of the current component and combine the licenses per the SPDX naming conventions.
*/
fn amalgamate_licenses(target_dir: &Path) -> SROutput {
    let output = SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    let mut license_str = String::new();
    let mut source_licenses: Vec<String> = Vec::new();
    let mut doc_licenses: Vec<String> = Vec::new();

    // Get the ordered listing of the component hierarchy
    let sr_entries = get_sr_paths(target_dir);

    // Compile the licenses of all the entries
    for entry in sr_entries {
        // We want the licenses from our current dot files
        let source_value = get_yaml_value(&entry, "source_license");
        let doc_value = get_yaml_value(&entry, "documentation_license");

        // Keep track of the license strings, avoiding duplicates
        if !source_licenses.contains(&source_value) {
            source_licenses.push(source_value);
        }
        if !doc_licenses.contains(&doc_value) {
            doc_licenses.push(doc_value);
        }
    }

    // Make sure everything is enclosed in parentheses
    license_str.push_str("(");

    // Step through all of the source licenses and append them to the license string
    let mut i = 0;
    for lic in source_licenses {
        // Make sure that the list is AND-concatenated
        if i > 0 {
            license_str.push_str(" AND ")
        }

        license_str.push_str(&lic);

        i = i + 1;
    }

    // Make sure that there's an AND concatenation after the source license
    if doc_licenses.len() > 0 && i > 0 {
        license_str.push_str(" AND ");
    }

    // Step through all of the documentation licenses and append them to the license string
    let mut j = 0;
    for lic in doc_licenses {
        // Make sure that the list is AND-concatenated
        if j > 0 {
            license_str.push_str(" AND ");
        }

        license_str.push_str(&lic);

        j = j + 1;
    }

    // Make sure everything is enclosed in parentheses
    license_str.push_str(")");

    update_json_value(&target_dir.join("package.json"), "license", &license_str);

    output
}

// Yields all the paths to .sr files in the target component's directory structure
fn get_sr_paths(target_dir: &Path) -> Vec<PathBuf> {
    let mut sr_paths = Vec::new();

    let walker = globwalk::GlobWalkerBuilder::from_patterns(target_dir, &[".sr"])
        .max_depth(100)
        .follow_links(false)
        .sort_by(path_cmp)
        .build()
        .expect("Could not build globwalk directory walker.")
        .into_iter()
        .filter_map(Result::ok);

    for sr_file in walker {
        sr_paths.push(sr_file.path().to_path_buf());
    }

    sr_paths
}

// Hackey way of comparing two paths by comparing them as strings, but is the only cross-platform way
// that gives a reliable ordering of the paths.
fn path_cmp(a: &walkdir::DirEntry, b: &walkdir::DirEntry) -> Ordering {
    let order: Ordering;

    if a.to_owned().into_path().to_string_lossy() < b.to_owned().into_path().to_string_lossy() {
        order = Ordering::Less;
    } else {
        order = Ordering::Greater;
    }

    order
}

/*
 * Extracts a value from a JSON file based on a string key.
*/
fn get_json_value(json_file: &PathBuf, key: &str) -> String {
    let mut value = String::new();

    // If the file doesn't exist, we can't do anything
    if json_file.exists() {
        // Open the file for reading
        let mut file = fs::File::open(&json_file).expect("Error opening JSON file.");

        // Attempt to read the contents of the file
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("ERROR: Unable to read the JSON file for this component");

        let lines = contents.lines();
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                let part: Vec<&str> = line.split(":").collect();
                value = part[1]
                    .replace("\"", "")
                    .replace(",", "")
                    .trim()
                    .to_string();
            }
        }
    } else {
        panic!(
            "JSON file {} not found, cannot extract data from it.",
            json_file.display()
        );
    }

    value
}

/*
 * Replaces the value corresponding to a key in a JSON file
*/
fn update_json_value(json_file: &PathBuf, key: &str, value: &str) {
    if json_file.exists() {
        // Open the file for reading
        let mut file = fs::File::open(&json_file).expect("Error opening JSON file.");

        // Attempt to read the contents of the component's .sr file
        let mut contents = String::new();
        let mut new_contents = String::new();
        file.read_to_string(&mut contents)
            .expect("ERROR: Unable to read the JSON file for this component");

        let lines = contents.lines();
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                // Grab the original value
                let part: Vec<&str> = line.split(":").collect();
                let old_value = part[1]
                    .replace("\"", "")
                    .replace(",", "")
                    .trim()
                    .to_string();

                // Scope the change to matching line and replace the original line with the new one
                let new_line = line.replace(&old_value, &value);
                new_contents = contents.replace(line, &new_line);
            }
        }

        // Make sure there's a change to write
        if !new_contents.is_empty() {
            // Try to write the contents back to the file
            fs::write(json_file, new_contents).expect("Could not write to JSON file.");
        }
    }
}

/*
 * Extracts a value from a yaml file based on a string key.
*/
fn get_yaml_value(yaml_file: &PathBuf, key: &str) -> String {
    let mut value = String::new();

    // If the file doesn't exist, we can't do anything
    if yaml_file.exists() {
        // Open the file for reading
        let mut file = fs::File::open(&yaml_file).expect("Error opening yaml file.");

        // Attempt to read the contents of the file
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("ERROR: Unable to read the yaml file for this component");

        let lines = contents.lines();
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                let part: Vec<&str> = line.split(":").collect();
                value = String::from(part[1].replace(",", "").trim());
            }
        }
    } else {
        panic!(
            "yaml file {} not found, cannot extract data from it.",
            yaml_file.display()
        );
    }

    value
}

/*
 * Replaces the value corresponding to a key in a yaml file
*/
fn update_yaml_value(yaml_file: &PathBuf, key: &str, value: &str) -> SROutput {
    let mut output = SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // Make sure the file even exists
    if yaml_file.exists() {
        let mut new_contents = String::new();

        // Read the entire contents of the file into a string so we can parse the lines
        let contents = match fs::read_to_string(yaml_file) {
            Ok(cont) => cont,
            Err(e) => {
                output.status = 4;
                output.stderr.push(format!(
                    "ERROR: Could not update the contents of the YAML file: {}",
                    e
                ));
                return output;
            }
        };

        // Step through all the lines in the file
        for line in contents.lines() {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                // Grab the original value
                let part: Vec<&str> = line.split(":").collect();
                let old_value = String::from(part[1].replace(",", "").trim());

                // Scope the change to matching line and replace the original line with the new one
                let new_line = line.replace(&old_value, &value);
                new_contents = contents.replace(line, &new_line);
            }
        }

        // Make sure there's a change to write
        if !new_contents.is_empty() {
            // Try to write the contents back to the file
            match fs::write(yaml_file, new_contents) {
                Ok(_) => (),
                Err(e) => {
                    output.status = 5;
                    output
                        .stderr
                        .push(format!("ERROR: Could not write to the YAML file: {}", e));
                    return output;
                }
            }; //.expect("Could not write to yaml file.");
        }
    } else {
        output.status = 3;
        output.stderr.push(String::from(
            "ERROR: YAML file to be updated does not exist.",
        ));
    }

    output
}

/*
 * Gets the parent directory of the current component
*/
fn get_parent_dir(target_dir: &Path) -> PathBuf {
    // Get the parent directory of this component's directory
    let parent_dir = target_dir
        .parent()
        .expect("ERROR: Could not get the parent directory of the target component.");

    parent_dir.to_path_buf()
}

/*
 * Gets the line ending that's appropriate for the OS we are running on.
 */
fn get_newline() -> String {
    let info = os_info::get();

    if info.os_type() == os_info::Type::Windows {
        String::from("\r\n")
    } else {
        String::from("\n")
    }
}

/*
 * Convenience function to combine the contents of two SROutput objects into one
 */
fn combine_sroutputs(mut dest: SROutput, src: SROutput) -> SROutput {
    // Collect the stdout values into one
    for line in src.stdout {
        dest.stdout.push(line);
    }

    // Collect the stderr values into one
    for line in src.stderr {
        dest.stderr.push(line);
    }

    // Make sure that if there was an error condition, we catch at least one of them
    // Runs the risk of masking one of the errors.
    if dest.status == 0 && src.status != 0 {
        dest.status = src.status;
    }

    dest
}

pub mod git_sr;
pub mod npm_sr;
pub mod templates;

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Component, Path};

    extern crate git2;
    extern crate uuid;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use std::process::Command;

    /*
     * Tests whether or not we can accurately find the parent dir of a component dir
     */
    #[test]
    fn test_get_parent_dir() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        assert!(&test_dir.join("toplevel").exists());
        assert_eq!(super::get_parent_dir(&test_dir.join("toplevel")), test_dir);
    }

    /*
     * Tests whether we can get and set yaml file properties correctly
     */
    #[test]
    fn test_yaml_file_handling() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Read the source license from the sample directory
        let source_license =
            super::get_yaml_value(&test_dir.join("toplevel").join(".sr"), "source_license");
        assert_eq!(source_license, "Unlicense");

        // Change the source license from the sample directory
        super::update_yaml_value(
            &test_dir.join("toplevel").join(".sr"),
            "source_license",
            "NotASourceLicense",
        );

        // Make sure the source license changed
        let source_license =
            super::get_yaml_value(&test_dir.join("toplevel").join(".sr"), "source_license");
        assert_eq!(source_license, "NotASourceLicense");

        // Read a non-existent key from the sample directory
        let value = super::get_yaml_value(&test_dir.join("toplevel").join(".sr"), "not_a_key");
        assert_eq!(value, "");
    }

    /*
     * Tests whether we can get and set json file properties correctly
     */
    #[test]
    fn test_json_file_handling() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Read the component name from the package.json file
        let name = super::get_json_value(&test_dir.join("toplevel").join("package.json"), "name");
        assert_eq!(name, "toplevel");

        // Change the component name in the package.json file
        super::update_json_value(
            &test_dir.join("toplevel").join("package.json"),
            "name",
            "NotAName",
        );

        // Make sure the component name changed in package.json
        let name = super::get_json_value(&test_dir.join("toplevel").join("package.json"), "name");
        assert_eq!(name, "NotAName");

        // Read a non-existent key from package.json
        let name =
            super::get_json_value(&test_dir.join("toplevel").join("package.json"), "not_a_key");
        assert_eq!(name, "");
    }

    /*
     * Tests whether or not the licenses are collected into the license field of package.json correctly.
     */
    #[test]
    fn test_amalgamate_licenses() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Make sure the license field starts with something other than the string we are looking for
        super::update_json_value(
            &test_dir.join("toplevel").join("package.json"),
            "license",
            "NotALicense",
        );

        super::amalgamate_licenses(&test_dir.join("toplevel"));

        // Make sure that all of the licenses were outlined correctly
        let license =
            super::get_json_value(&test_dir.join("toplevel").join("package.json"), "license");

        assert_eq!(
            license,
            "(Unlicense AND NotASourceLicense AND CC0-1.0 AND NotADocLicense AND CC-BY-4.0)"
        );
    }

    #[test]
    fn test_get_licenses() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Make sure that we get the proper licenses back when requested
        let licenses = super::get_licenses(&test_dir);

        assert_eq!(licenses.0, "Unlicense");
        assert_eq!(licenses.1, "CC0-1.0");
    }

    #[test]
    fn test_list_all_licenses() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Make suer that we get a proper license listing when requested
        let license_listing = super::list_all_licenses(&test_dir.join("toplevel"));

        assert!(license_listing.contains("Licenses Specified In This Component:"));
        assert!(license_listing.contains("Unlicense"));
        assert!(license_listing.contains("CC0-1.0"));
        assert!(license_listing.contains("NotASourceLicense"));
        assert!(license_listing.contains("NotADocLicense"));
        assert!(license_listing.contains("CC-BY-4.0"));
    }

    #[test]
    fn test_gitignore_template() {
        let content = super::templates::gitignore_template();

        assert!(content.contains("# Dependency directories"));
        assert!(content.contains("node_modules/"));
        assert!(content.contains("# Distribution directory"));
        assert!(content.contains("dist/"));

        // Render the template and make sure we got what was expected
        let mut globals = liquid::value::Object::new();

        let render = super::render_template(".gitignore.liquid", &mut globals);

        assert!(render.contains("# Dependency directories"));
        assert!(render.contains("node_modules/"));
        assert!(render.contains("# Distribution directory"));
        assert!(render.contains("dist/"));
    }

    #[test]
    fn test_sr_file_template() {
        let content = super::templates::sr_file_template();

        assert!(content.contains("source_license: {{source_license}},"));
        assert!(content.contains("documentation_license: {{doc_license}}"));

        // Render the template and make sure we got was expected
        let mut globals = liquid::value::Object::new();
        globals.insert(
            "source_license".into(),
            liquid::value::Value::scalar("NotASourceLicense"),
        );
        globals.insert(
            "doc_license".into(),
            liquid::value::Value::scalar("NotADocLicense"),
        );

        let render = super::render_template(".sr.liquid", &mut globals);

        assert!(render.contains("source_license: NotASourceLicense,"));
        assert!(render.contains("documentation_license: NotADocLicense"));
    }

    #[test]
    fn test_bom_data_yaml_template() {
        let content = super::templates::bom_data_yaml_template();

        assert!(content.contains("# Bill of Materials Data for {{name}}"));
        assert!(content.contains("parts:"));
        assert!(content.contains("    - specific_component_variation"));
        assert!(content.contains("    notes: ''"));
        assert!(content.contains("order:"));
        assert!(content.contains("  -component_1"));

        // Render the template and make sure we got was expected
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar("TopLevel"));

        let render = super::render_template("bom_data.yaml.liquid", &mut globals);

        assert!(render.contains("# Bill of Materials Data for TopLevel"));
        assert!(render.contains("parts:"));
        assert!(render.contains("    - specific_component_variation"));
        assert!(render.contains("    notes: ''"));
        assert!(render.contains("order:"));
        assert!(render.contains("  -component_1"));
    }

    #[test]
    fn test_package_json_template() {
        let content = super::templates::package_json_template();

        assert!(content.contains("  \"name\": \"{{name}}\","));
        assert!(content.contains("  \"license\": \"{{license}}\","));

        // Render the template and make sure we got was expected
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar("TopLevel"));
        globals.insert(
            "license".into(),
            liquid::value::Value::scalar("(NotASourceLicense AND NotADocLicense)"),
        );

        let render = super::render_template("package.json.liquid", &mut globals);

        assert!(render.contains("  \"name\": \"TopLevel\","));
        assert!(render.contains("  \"license\": \"(NotASourceLicense AND NotADocLicense)\","));
    }

    #[test]
    fn test_readme_template() {
        let content = super::templates::readme_template();

        assert!(content.contains("# {{name}}"));
        assert!(content.contains("Developed in [Sliderule](http://sliderule.io) an implementation of the [Distributed OSHW Framework](http://dof.sliderule.io)."));

        // Render the template and make sure we got was expected
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar("TopLevel"));

        let render = super::render_template("README.md.liquid", &mut globals);

        assert!(render.contains("# TopLevel"));
        assert!(render.contains("Developed in [Sliderule](http://sliderule.io) an implementation of the [Distributed OSHW Framework](http://dof.sliderule.io)."));
    }

    #[test]
    fn test_generate_dot_file() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir).expect("Could not create temporary directory for test.");

        super::generate_dot_file(&temp_dir, "NotASourceLicense", "NotADocLicense");

        let mut file = fs::File::open(&temp_dir.join(".sr")).expect("Unable to open the sr file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the sr file");

        assert!(contents.contains("source_license: NotASourceLicense,"));
        assert!(contents.contains("documentation_license: NotADocLicense"));
    }

    #[test]
    fn test_generate_gitignore() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir).expect("Could not create temporary directory for test.");

        super::generate_gitignore(&temp_dir);

        let mut file = fs::File::open(&temp_dir.join(".gitignore"))
            .expect("Unable to open the gitignore file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the gitignore file");

        assert!(contents.contains("node_modules/"));
        assert!(contents.contains("dist/"));
    }

    #[test]
    fn test_generate_package_json() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir).expect("Could not create temporary directory for test.");

        super::generate_package_json(&temp_dir, "TopLevel", "NotASourceLicense");

        let mut file = fs::File::open(&temp_dir.join("package.json"))
            .expect("Unable to open the package.json file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the package.json file");

        assert!(contents.contains("  \"name\": \"TopLevel\","));
        assert!(contents.contains("  \"license\": \"NotASourceLicense\","));
    }

    #[test]
    fn test_generate_bom() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir).expect("Could not create temporary directory for test.");

        super::generate_bom(&temp_dir, "TopLevel");

        let mut file = fs::File::open(&temp_dir.join("bom_data.yaml"))
            .expect("Unable to open the bom_data.yaml file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the package.json file");

        assert!(contents.contains("# Bill of Materials Data for TopLevel"));
    }

    #[test]
    fn test_generate_readme() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir).expect("Could not create temporary directory for test.");

        super::generate_readme(&temp_dir, "TopLevel");

        let mut file =
            fs::File::open(&temp_dir.join("README.md")).expect("Unable to open the README.md file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the package.json file");

        assert!(contents.contains("# TopLevel"));
    }

    #[test]
    fn test_update_local_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let output = super::update_local_component(&test_dir.join("toplevel"));

        // We should not have gotten an error
        assert_eq!(0, output.status);

        assert_eq!(output.stdout[0].trim(), "Already up to date.");
        assert_eq!(output.stdout[1], "Component updated successfully.");
    }

    #[test]
    fn test_update_dependencies() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let output = super::update_dependencies(&test_dir.join("toplevel"));

        // We should not have gotten an error
        assert_eq!(0, output.status);

        assert!(output.stdout[1].contains("Dependencies were updated successfully."));
    }

    #[test]
    fn test_download_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let output = super::download_component(
            &test_dir.join("toplevel"),
            "https://github.com/jmwright/toplevel.git",
        );

        // We should not have gotten an error
        assert_eq!(0, output.status);

        assert!(output.stdout[1].contains("Component was downloaded successfully."));
    }

    #[test]
    fn test_remove_remote_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Set up a cache directory to keep the system npm cache from getting messed up by the tests
        let cache_dir = temp_dir.join(format!("cache_{}", uuid::Uuid::new_v4()));

        let output = super::remove_remote_component(
            &test_dir.join("toplevel"),
            "blink_firmware",
            Some(cache_dir.to_string_lossy().to_string()),
        );

        // We should not have gotten an error
        assert_eq!(0, output.status);

        assert!(!test_dir
            .join("toplevel")
            .join("node_modules")
            .join("blink_firmware")
            .exists());
    }

    #[test]
    fn test_add_remote_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Set up a cache directory to keep the system npm cache from getting messed up by the tests
        let cache_dir = temp_dir.join(format!("cache_{}", uuid::Uuid::new_v4()));

        let output = super::add_remote_component(
            &test_dir.join("toplevel"),
            "https://github.com/jmwright/arduino-sr.git",
            Some(cache_dir.to_string_lossy().to_string()),
        );

        let component_path = test_dir
            .join("toplevel")
            .join("node_modules")
            .join("arduino-sr");

        // We should not have gotten an error
        assert_eq!(0, output.status);

        // The arduino-sr directory should exist
        assert!(component_path.exists());

        // The arduino-sr directory should be a valid component
        assert!(is_valid_component(
            &component_path,
            "arduino-sr",
            "Unlicense",
            "CC0-1.0"
        ));
    }

    #[test]
    fn test_change_licenses() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let output = super::change_licenses(
            &test_dir.join("toplevel"),
            String::from("TestSourceLicense"),
            String::from("TestDocLicense"),
        );

        // We should not have gotten an error
        assert_eq!(0, output.status);
        assert!(output.stderr.is_empty());

        // Make sure that the package.json file license was changed
        assert!(file_contains_content(
            &test_dir.join("toplevel").join("package.json"),
            9999,
            "TestSourceLicense",
        ));
        assert!(file_contains_content(
            &test_dir.join("toplevel").join("package.json"),
            9999,
            "TestDocLicense",
        ));
        // Check to make sure the licenses were actually changed
        assert!(file_contains_content(
            &test_dir.join("toplevel").join(".sr"),
            9999,
            "source_license: TestSourceLicense,"
        ));
        assert!(file_contains_content(
            &test_dir.join("toplevel").join(".sr"),
            9999,
            "documentation_license: TestDocLicense"
        ));
    }

    #[test]
    fn test_remove() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Remove a local component so we can test it
        let output = super::remove(&test_dir.join("toplevel"), "level1");

        // We should not have gotten an error
        assert_eq!(0, output.status);
        assert!(output.stderr.is_empty());

        // Make sure that the level1 directory was removed
        assert!(!&test_dir
            .join("toplevel")
            .join("components")
            .join("level1")
            .exists());

        // Remove a remote component so we can test it
        let output = super::remove(&test_dir.join("toplevel"), "blink_firmware");

        // We should not have gotten an error
        assert_eq!(0, output.status);

        // Make sure that the level1 directory was removed
        assert!(!&test_dir
            .join("toplevel")
            .join("node_modules")
            .join("level1")
            .exists());
    }

    #[test]
    fn test_create_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Generate a new component
        let output = super::create_component(
            &test_dir,
            String::from("nextlevel"),
            String::from("TestSourceLicense"),
            String::from("TestDocLicense"),
        );

        // We should not have gotten an error
        assert_eq!(0, output.status);

        // We should have gotten a message that the component was finished being set up
        assert_eq!(
            "Finished setting up component.",
            output.stdout[output.stdout.len() - 1]
        );

        // We should have a valid component when all is said and done
        assert!(is_valid_component(
            &test_dir.join("nextlevel"),
            "nextlevel",
            "TestSourceLicense",
            "TestDocLicense"
        ));
    }

    #[test]
    fn test_refactor() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let demo_dir = test_dir.join("demo");
        let remote_dir = demo_dir.join("remote");

        // Create the demo directory
        fs::create_dir(&demo_dir).expect("Failed to create demo directory.");

        Command::new("git")
            .args(&["init", "--bare"])
            .current_dir(&demo_dir)
            .output()
            .expect("failed to initialize bare git repository in demo directory");

        // Create the remote directory for the nextlevel project
        fs::create_dir(&remote_dir).expect("Failed to create top component directory.");

        Command::new("git")
            .args(&["init", "--bare"])
            .current_dir(&remote_dir)
            .output()
            .expect("failed to initialize bare git repository in demo directory");

        // Start a new git daemon server in the current remote repository
        Command::new("git")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .args(&[
                "daemon",
                "--reuseaddr",
                "--export-all",
                "--base-path=.",
                "--verbose",
                "--enable=receive-pack",
                ".",
            ])
            .current_dir(demo_dir)
            .spawn()
            .expect("ERROR: Could not launch git daemon.");

        // Generate a new component
        let output = super::create_component(
            &test_dir.join("toplevel"),
            String::from("remote"),
            String::from("TestSourceLicense"),
            String::from("TestDocLicense"),
        );

        // Make sure the new directory exists and is a valid component
        assert!(is_valid_component(
            &test_dir.join("toplevel").join("components").join("remote"),
            "remote",
            "TestSourceLicense",
            "TestDocLicense"
        ));

        // Make sure we did not get any errors
        assert_eq!(0, output.stderr.len());

        let output = super::refactor(
            &test_dir.join("toplevel"),
            String::from("remote"),
            String::from("git://127.0.0.1/remote"),
            None,
            None,
        );

        if output.stderr.len() > 0 {
            for out in &output.stderr {
                println!("{:?}", out);
            }
        }

        assert_eq!(
            "Finished refactoring local component to remote repository.",
            output.stdout[output.stdout.len() - 1]
        );

        // Make sure the component was reinstalled in the node_modules directory
        assert!(is_valid_component(
            &test_dir
                .join("toplevel")
                .join("node_modules")
                .join("remote"),
            "remote",
            "TestSourceLicense",
            "TestDocLicense"
        ));

        // Make sure there are no git processes left around after we're done
        kill_git();
    }

    #[test]
    fn test_upload_component() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let demo_dir = test_dir.join("demo");
        let remote_dir = demo_dir.join("nextlevel");

        // Create the demo directory
        fs::create_dir(&demo_dir).expect("Failed to create demo directory.");

        Command::new("git")
            .args(&["init", "--bare"])
            .current_dir(&demo_dir)
            .output()
            .expect("failed to initialize bare git repository in demo directory");

        // Create the remote directory for the nextlevel project
        fs::create_dir(&remote_dir).expect("Failed to create top component directory.");

        Command::new("git")
            .args(&["init", "--bare"])
            .current_dir(&remote_dir)
            .output()
            .expect("failed to initialize bare git repository in demo directory");

        // Start a new git daemon server in the current remote repository
        Command::new("git")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .args(&[
                "daemon",
                "--reuseaddr",
                "--export-all",
                "--base-path=.",
                "--verbose",
                "--enable=receive-pack",
                ".",
            ])
            .current_dir(demo_dir)
            .spawn()
            .expect("ERROR: Could not launch git daemon.");

        // Generate a new component
        let output = super::create_component(
            &test_dir,
            String::from("nextlevel"),
            String::from("TestSourceLicense"),
            String::from("TestDocLicense"),
        );

        // Make sure we did not get any errors
        assert_eq!(0, output.stderr.len());

        let output = super::upload_component(
            &test_dir.join("nextlevel"),
            String::from("Initial commit"),
            String::from("git://127.0.0.1/nextlevel"),
            None,
            None,
        );

        if output.stderr.len() > 0 {
            for out in &output.stderr {
                println!("{:?}", out);
            }
        }

        assert_eq!(
            "Done uploading component.",
            output.stdout[output.stdout.len() - 1]
        );
        assert_eq!(
            "Changes pushed using git.",
            output.stdout[output.stdout.len() - 2]
        );

        // To test properly, we have to re-download the component and check if it's valid
        let output = super::download_component(
            &test_dir.join("toplevel"),
            &String::from("git://127.0.0.1/nextlevel"),
        );

        if output.stderr.len() > 0 {
            for out in &output.stderr {
                println!("{:?}", out);
            }
        }

        assert!(is_valid_component(
            &test_dir.join("toplevel").join("nextlevel"),
            "nextlevel",
            "TestSourceLicense",
            "TestDocLicense"
        ));

        // Make sure there are no git processes left around after we're done
        kill_git();
    }

    #[test]
    fn test_get_sr_paths() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        let sr_paths = super::get_sr_paths(&test_dir.join("toplevel"));

        // This is in here to help us troubleshoot if this test fails on one of the CI OSes
        for sr_path in &sr_paths {
            println!("{:?}", sr_path);
        }

        let path_parts = sr_paths[0].components().collect::<Vec<_>>();
        assert_eq!(
            path_parts[path_parts.len() - 1],
            Component::Normal(OsStr::new(".sr"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 2],
            Component::Normal(OsStr::new("toplevel"))
        );

        let path_parts = sr_paths[1].components().collect::<Vec<_>>();
        assert_eq!(
            path_parts[path_parts.len() - 1],
            Component::Normal(OsStr::new(".sr"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 2],
            Component::Normal(OsStr::new("level1"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 3],
            Component::Normal(OsStr::new("components"))
        );

        let path_parts = sr_paths[2].components().collect::<Vec<_>>();
        assert_eq!(
            path_parts[path_parts.len() - 1],
            Component::Normal(OsStr::new(".sr"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 2],
            Component::Normal(OsStr::new("level2"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 3],
            Component::Normal(OsStr::new("components"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 4],
            Component::Normal(OsStr::new("level1"))
        );

        let path_parts = sr_paths[3].components().collect::<Vec<_>>();
        assert_eq!(
            path_parts[path_parts.len() - 1],
            Component::Normal(OsStr::new(".sr"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 2],
            Component::Normal(OsStr::new("level3"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 3],
            Component::Normal(OsStr::new("components"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 4],
            Component::Normal(OsStr::new("level2"))
        );

        let path_parts = sr_paths[4].components().collect::<Vec<_>>();
        assert_eq!(
            path_parts[path_parts.len() - 1],
            Component::Normal(OsStr::new(".sr"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 2],
            Component::Normal(OsStr::new("blink_firmware"))
        );
        assert_eq!(
            path_parts[path_parts.len() - 3],
            Component::Normal(OsStr::new("node_modules"))
        );
    }

    #[test]
    fn test_get_version() {
        let version_num = super::get_version();

        assert_eq!(version_num, "0.2.1");
    }

    // Cleans up the git daemon processes after tests run
    fn kill_git() {
        let info = os_info::get();

        if info.os_type() == os_info::Type::Windows {
            //taskkill /f /t /im wwahost.exe
            Command::new("taskkill")
                .args(&["/f", "/t", "/im", "git"])
                .output()
                .unwrap();

            Command::new("taskkill")
                .args(&["/f", "/t", "/im", "git-daemon"])
                .output()
                .unwrap();
            Command::new("taskkill")
                .args(&["/f", "/t", "/im", "git.exe"])
                .output()
                .unwrap();

            Command::new("taskkill")
                .args(&["/f", "/t", "/im", "git-daemon.exe"])
                .output()
                .unwrap();
        } else {
            Command::new("killall")
                .args(&["git"])
                .output()
                .expect("failed to kill git process");
        }
    }

    /*
     * Sets up a test directory for our use.
     */
    fn set_up(temp_dir: &PathBuf, dir_name: &str) -> PathBuf {
        let url = "https://github.com/jmwright/toplevel.git";

        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);

        // Create the temporary test directory
        fs::create_dir(temp_dir.join(&test_dir_name))
            .expect("Unable to create temporary directory.");

        match git2::Repository::clone(&url, temp_dir.join(&test_dir_name).join(dir_name)) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };

        temp_dir.join(test_dir_name)
    }

    /*
     * Tests if a directory has the correct contents to be a component.
     */
    fn is_valid_component(
        component_path: &Path,
        component_name: &str,
        source_license: &str,
        doc_license: &str,
    ) -> bool {
        let mut is_valid = true;

        // Make sure that the component path exists
        if !component_path.exists() {
            is_valid = false;
            println!(
                "The component directory {:?} does not exist",
                component_path
            );
        } else {
            let paths = fs::read_dir(component_path).unwrap();

            for path in paths {
                println!("Component path: {}", path.unwrap().path().display());
            }
        }

        // Make sure the BoM data file exists
        if !component_path.join("bom_data.yaml").exists() {
            is_valid = false;
            println!(
                "The file {:?}/bom_data.yaml does not exist.",
                component_path
            );
        }

        // Make sure the component directory exists
        if !component_path.join("components").exists() {
            is_valid = false;
            println!(
                "The directory {:?}/components does not exist.",
                component_path
            );
        }

        // Make sure the docs directory exists
        if !component_path.join("docs").exists() {
            is_valid = false;
            println!("The directory {:?}/docs does not exist.", component_path);
        }

        // Make sure the package.json file exists
        if !component_path.join("package.json").exists() {
            is_valid = false;
            println!("The file {:?}/package.json does not exist.", component_path);
        }

        // Make sure the README.md file exists
        if !component_path.join("README.md").exists() {
            is_valid = false;
            println!("The file {:?}/README.md does not exist.", component_path);
        }

        // Make sure the source directory exists
        if !component_path.join("source").exists() {
            is_valid = false;
            println!("The directory {:?}/source does not exist.", component_path);
        }

        let bom_file = component_path.join("bom_data.yaml");
        let package_file = component_path.join("package.json");
        let readme_file = component_path.join("README.md");
        let dot_file = component_path.join(".sr");

        // Check the content of the files and directories as appropriate here
        if !file_contains_content(
            &bom_file,
            0,
            &format!("# Bill of Materials Data for {}", component_name),
        ) {
            is_valid = false;
            println!(
                "The bill to materials file in {:?} does not contain the correct header.",
                component_path
            );
        }
        if !file_contains_content(&bom_file, 12, "-component_1") {
            is_valid = false;
            println!("The bill to materials file in {:?} does not contain the '-component_1' entry in the right place.", component_path);
        }
        if !file_contains_content(
            &package_file,
            9999,
            &format!("\"name\": \"{}\",", component_name),
        ) {
            is_valid = false;
            println!("The package.json file in {:?} does not contain the component name entry in the right place.", component_path);
        }
        if !file_contains_content(
            &package_file,
            9999,
            &format!("\"license\": \"({} AND {})\",", source_license, doc_license),
        ) {
            is_valid = false;
            println!("The package.json file in {:?} does not contain the the correct license entry in the right place.", component_path);
        }
        if !file_contains_content(&readme_file, 0, &format!("# {}", component_name)) {
            is_valid = false;
            println!("The README.md file in {:?} does not contain the the correct header entry in the right place.", component_path);
        }
        if !file_contains_content(&readme_file, 1, "New Sliderule component.") {
            is_valid = false;
            println!("The README.md file in {:?} does not contain the the correct Sliderule mention in the right place.", component_path);
        }
        if !file_contains_content(
            &dot_file,
            0,
            &format!("source_license: {},", source_license),
        ) {
            is_valid = false;
            println!(
                "The .sr file in {:?} does not contain the the correct source license in the right place.",
                component_path
            );
        }
        if !file_contains_content(
            &dot_file,
            1,
            &format!("documentation_license: {}", doc_license),
        ) {
            is_valid = false;
            println!("The .sr file in {:?} does not contain the the correct documentation license in the right place.", component_path);
        }

        is_valid
    }

    /*
     * Helper function that checks to make sure that given text is present in the files.
     */
    fn file_contains_content(file_path: &Path, line: usize, text: &str) -> bool {
        let contains_content: bool;

        // Read the contents of the file
        let contents =
            fs::read_to_string(file_path).expect("ERROR: Cannot read the contents of the file.");

        // See if the user just wants to make sure the content is somewhere in the file
        if line == 9999 {
            contains_content = contents.contains(text);
        } else {
            // Break the file down into something we can index
            let contents: Vec<&str> = contents.lines().collect();

            // See if the line we are interested in is exactly the content specified
            contains_content = contents[line].trim() == text;
        }

        contains_content
    }
}
