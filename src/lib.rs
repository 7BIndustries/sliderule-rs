#![allow(dead_code)]

extern crate liquid;
extern crate walkdir;
extern crate os_info;
extern crate git2;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

pub struct SROutput {
    pub status: i32,
    pub wrapped_status: i32,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>
}


/*
 * Create a new Sliderule component or convert an existing project to being a Sliderule project.
*/
pub fn create_component(target_dir: &Path, name: &String, source_license: &String, doc_license: &String) {
    // let source_license: String;
    // let doc_license: String;

    // The path can either lead to a top level component (project), or a component nested within a project
    let component_dir: PathBuf;

    // This is a top level component (project)
    if target_dir.join(".sr").exists() {
        component_dir = target_dir.join("components").join(name);
    }
    else {
        component_dir = target_dir.join(name);
    }

    // Create a directory for our component
    fs::create_dir(&component_dir)
        .expect("Could not create dist directory.");

    // Make a new directory in components, cd into it, and then run the rest of this code
    env::set_current_dir(&component_dir)
        .expect("Could not change into components directory.");

    // Create the components directory, if needed
    if !target_dir.join("components").exists() {
        fs::create_dir(target_dir.join("components"))
            .expect("Could not create components directory.");
    }
    else {
        println!("components directory already exists, using existing directory.");
    }

    // Create the dist directory, if needed
    if !target_dir.join("dist").exists() {
        fs::create_dir(target_dir.join("dist"))
            .expect("Could not create dist directory.");
    }
    else {
        println!("dist directory already exists, using existing directory.");
    }

    // Create the docs directory, if needed
    if !target_dir.join("docs").exists() {
        fs::create_dir(target_dir.join("docs"))
            .expect("Could not create docs directory.");
    }
    else {
        println!("docs directory already exists, using existing directory.");
    }

    //Create the source directory, if needed
    if !target_dir.join("source").exists() {
        fs::create_dir(target_dir.join("source"))
            .expect("Could not create source directory.");
    }
    else {
        println!("source directory already exists, using existing directory.");
    }

    // Generate the template readme file
    generate_readme(&target_dir, &name);

    // Generate bom_data.yaml
    generate_bom(&target_dir, &name);

    // Generate package.json, if needed
    generate_package_json(&target_dir, &name, &source_license);

    // Generate the .sr file that provides extra information about this component
    generate_dot_file(&target_dir, &source_license, &doc_license);

    // Make sure that our package.json file is updated with all the license info
    amalgamate_licenses(&target_dir);

    println!("Finished setting up component.");
}


/*
 * Uploads any changes to the project to the remote repository.
*/
pub fn upload_component(target_dir: &Path, message: String, url: &String) {
    // Make sure that our package.json file is updated with all the license info
    amalgamate_licenses(&target_dir);

    // Initialize as a repo only if needed
    if !target_dir.join(".git").exists() {
        // Initialize the git repository and set the remote URL to push to
        git_sr::git_init(target_dir, &url);
    }

    // Create the gitignore file only if we need to
    if !target_dir.join(".gitignore").exists() {
        // Generate gitignore file so that we don't commit and push things we shouldn't be
        generate_gitignore(&target_dir);
    }
    
    // Add all changes, commit and push
    git_sr::git_add_and_commit(target_dir, message);

    println!("Done uploading component.");
}


/*
 * Converts a local component into a remote component, asking for a remote repo to push it to.
*/
pub fn refactor(target_dir: &Path, name: String, url: String) {
    let component_dir = target_dir.join("components").join(&name);

    if component_dir.exists() {
        // Upload the current component to the remote repo
        upload_component(&target_dir, String::from("Initial commit, refactoring component"), &url);

        // Remove the local component and then install it from the remote using npm
        remove(&target_dir, &name);
        add_remote_component(&target_dir, &url);

        // Shouldn't need it here, but make sure that our package.json file is updated with all the license info
        amalgamate_licenses(&target_dir);
    }
    else {
        panic!("ERROR: The component does not exist in the components directory.");
    }

    println!("Finished refactoring local component to remote repository.");
}


/*
 * Removes a component from the project structure.
*/
pub fn remove(target_dir: &Path, name: &str) {
    let component_dir = target_dir.join("components").join(name);

    // If the component exists as a subdirectory of components delete the directory directly otherwise use npm to remove it.
    if component_dir.exists() {
        println!("Deleting component directory.");

        // Step through every file and directory in the path to be deleted and make sure that none are read-only
        for entry in walkdir::WalkDir::new(&component_dir) {
            let entry = entry
                .expect("Could not handle entry while walking components directory tree.");

            // Remove read-only permissions on every entry
            let md = &entry.path().metadata().
                expect("ERROR: Could not get metadata.");
            let mut perms = md.permissions();
            perms.set_readonly(false);
            fs::set_permissions(&entry.path(), perms)
                .expect("Error: Failed to set permissions on .git directory");
        }

        fs::remove_dir_all(component_dir)
            .expect("ERROR: not able to delete component directory.");
    }
    else {
        remove_remote_component(&target_dir, name);
    }

    // Make sure that our package.json file is updated with all the license info
    amalgamate_licenses(&target_dir);

    println!("{} component removed.", name);
}

/*
 * Allows the user to change the source and/or documentation licenses for the project.
*/
pub fn change_licenses(target_dir: &Path, source_license: &String, doc_license: &String) {
    let cwd = target_dir.join(".sr");

    update_yaml_value(&cwd, "source_license", source_license);
    update_yaml_value(&cwd, "documentation_license", doc_license);

    // Make sure our new licenses are up to date in package.json
    amalgamate_licenses(&target_dir);
}

/*
 * Adds a remote component via URL to node_modules.
*/
pub fn add_remote_component(target_dir: &Path, url: &str) {
    npm_sr::npm_install(target_dir, &url);

    // Make sure that our package.json file is updated with all the license info
    amalgamate_licenses(&target_dir);
}

/*
 * Removes a remote component via the name.
 */
pub fn remove_remote_component(target_dir: &Path, name: &str) {
    // Use npm to remove the remote component
    npm_sr::npm_uninstall(target_dir, name);
}

/*
 * Downloads (copies) a component from a remote repository.
*/
pub fn download_component(target_dir: &Path, url: &str) {
    git_sr::git_clone(target_dir, url);
}

/*
    * Updates all remote components in node_modules
    */
pub fn update_dependencies(target_dir: &Path) -> SROutput {
    let mut output = npm_sr::npm_install(target_dir, "");

    if output.status != 0 || output.wrapped_status != 0 {
        output.stderr.push(String::from("ERROR: Dependencies were not successfully updated"));
    }
    
    if output.status == 0 && output.wrapped_status == 0 {
        output.stdout.push(String::from("Dependencies were updated successfully."));
    }

    // Make sure that our package.json file is updated with all the license info
    amalgamate_licenses(&target_dir);

    output
}

/*
 * Updates the local component who's directory we're in
*/
pub fn update_local_component(target_dir: &Path) ->  SROutput {
    let mut output = SROutput {status: 0, wrapped_status: 0, stderr: Vec::new(), stdout: Vec::new()};

    if target_dir.join(".git").exists() {
        output = git_sr::git_pull(target_dir);

        // Make sure that our package.json file is updated with all the license info
        amalgamate_licenses(&target_dir);

        // Give the user an idea of whether the update was successful or not
        if output.status == 0 {
            output.stdout.push(String::from("Component updated successfully."));
        }
        else {
            output.stdout.push(String::from("Component not updated successfully."));
        }
    }
    else {
        output.status = 1;
        output.stderr.push(String::from("ERROR: Component is not set up as a repository, cannot update it."));
    }

    output
}


/*
 * Generates a template README.md file to help the user get started.
*/
fn generate_readme(target_dir: &Path, name: &str) {
    if !target_dir.join("README.md").exists() {
        // Add the things that need to be put substituted into the README file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

        let contents = render_template("README.md.liquid", &mut globals);

        // Write the template text into the readme file
        fs::write(target_dir.join("README.md"), contents)
            .expect("Could not write to README.md file.");
    }
    else {
        println!("README.md already exists, using existing file and refusing to overwrite.");
    }
}


/*
 * Generates a bill of materials from a template.
*/
fn generate_bom(target_dir: &Path, name: &str) {
    if !target_dir.join("bom_data.yaml").exists() {
        // Add the things that need to be put substituted into the BoM file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

        let contents = render_template("bom_data.yaml.liquid", &mut globals);

        // Write the template text into the readme file
        fs::write(target_dir.join("bom_data.yaml"), contents)
            .expect("Could not write to bom_data.yaml.");
    }
    else {
        println!("bom_data.yaml already exists, using existing file and refusing to overwrite.");
    }
}


/*
 * Generates a package.json file for npm based on a Liquid template.
*/
fn generate_package_json(target_dir: &Path, name: &str, license: &str) {
    if !target_dir.join("package.json").exists() {
        // Add the things that need to be put substituted into the package file
        let mut globals = liquid::value::Object::new();
        globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));
        globals.insert("license".into(), liquid::value::Value::scalar(license.to_owned()));

        let contents = render_template("package.json.liquid", &mut globals);

        // Write the contents into the file
        fs::write(target_dir.join("package.json"), contents)
            .expect("Could not write to package.json.");
    }
    else {
        println!("package.json already exists, using existing file and refusing to overwrite.");
    }
}


/*
 * Generates the .gitignore file used by the git command to ignore files and directories.
*/
fn generate_gitignore(target_dir: &Path) {
    if !target_dir.join(".gitignore").exists() {
        // Add the things that need to be put substituted into the gitignore file (none at this time)
        let mut globals = liquid::value::Object::new();

        let contents = render_template(".gitignore.liquid", &mut globals);

        // Write the contents to the file
        fs::write(target_dir.join(".gitignore"), contents)
            .expect("Could not write to .gitignore.");
    }
    else {
        println!(".gitignore already exists, using existing file and refusing to overwrite.");
    }
}


/*
 * Generates the dot file that tracks whether this is a top level component/project or a sub-component
*/
fn generate_dot_file(target_dir: &Path, source_license: &str, doc_license: &str) {
    if !target_dir.join(".sr").exists() {
        // Add the things that need to be put substituted into the .top file (none at this time)
        let mut globals = liquid::value::Object::new();
        globals.insert("source_license".into(), liquid::value::Value::scalar(source_license.to_owned()));
        globals.insert("doc_license".into(), liquid::value::Value::scalar(doc_license.to_owned()));

        let contents = render_template(".sr.liquid", &mut globals);

        // Write the contents to the file
        fs::write(target_dir.join(".sr"), contents)
            .expect("Could not write to .sr file.");
    }
    else {
        println!(".sr already exists, using existing file and refusing to overwrite.");
    }
}


/*
 * Reads a template to a string so that it can be written to a new components directory structure.
*/
fn render_template(template_name: &str, globals: &mut liquid::value::Object) -> String {
    let mut contents = String::new();

    if template_name == ".sr.liquid" {
        contents = templates::sr_file_template();
    }
    else if template_name == ".gitignore.liquid" {
        contents = templates::gitignore_template();
    }
    else if template_name == "bom_data.yaml.liquid" {
        contents = templates::bom_data_yaml_template();
    }
    else if template_name == "package.json.liquid" {
        contents = templates::package_json_template();
    }
    else if template_name == "README.md.liquid" {
        contents = templates::readme_template();
    }

    // Render the output of the template using Liquid
    let template = liquid::ParserBuilder::with_liquid()
        .build()
        .parse(&contents)
            .expect("Could not parse template using Liquid.");

    let output = template.render(globals)
        .expect("Could not render template using Liquid.");

    output
}

/*
 * Prints out each of the licenses in the component's directory tree so that
 * users can see where the licenses reside.
*/
pub fn list_all_licenses(target_dir: &Path) -> String {
    let nl = get_newline();
    let mut license_listing = String::from("Licenses Specified In This Component:");
    license_listing.push_str(&nl);

    // Walk through every sub-directory in this component, looking for .sr files
    for entry in walkdir::WalkDir::new(&target_dir) {
        let entry = entry
            .expect("Could not handle entry while walking components directory tree.");

        // If we have a .sr file, keep it for later license extraction
        if entry.path().ends_with(".sr") {
            // The current component path
            let component_path = entry.path().parent()
                .expect("Could not get the parent path of the .sr file.");

            // We want the licenses from our current dot files
            let source_value = get_yaml_value(&entry.path().to_path_buf(), "source_license");
            let doc_value = get_yaml_value(&entry.path().to_path_buf(), "documentation_license");

            license_listing.push_str(&format!("Path: {}, Source License: {}, Documentation License: {}{}", component_path.display(), source_value, doc_value, nl));
        }
    }

    license_listing
}

/*
 * Extracts the source and documentation licenses from a component's .sr file.
*/
pub fn get_licenses(target_dir: &Path) -> (String, String) {
    let sr_file: PathBuf;

    // We can hand back the default licenses, if nothing else
    let mut source_license = String::from("Unlicense");
    let mut doc_license = String::from("CC0-1.0");

    // If we're in a component directory, pull the license info from that
    sr_file = target_dir.join(".sr");

    // Safety check to make sure the file exists
    if sr_file.exists() {
        println!("Attempting to extract license from {}", sr_file.display());

        // Extract the licenses from the file
        source_license = get_yaml_value(&sr_file, "source_license");
        doc_license = get_yaml_value(&sr_file, "documentation_license");
    }

    (source_license, doc_license)
}

/*
 * Walk the directory structure of the current component and combine the licenses per the SPDX naming conventions.
*/
fn amalgamate_licenses(target_dir: &Path) {
    let mut license_str = String::new();
    let mut source_licenses: Vec<String> = Vec::new();
    let mut doc_licenses: Vec<String> = Vec::new();

    // Walk through every sub-directory in this component, looking for .sr files
    for entry in walkdir::WalkDir::new(&target_dir) {
        let entry = entry
            .expect("Could not handle entry while walking components directory tree.");

        // If we have a .sr file, keep it for later license extraction
        if entry.path().ends_with(".sr") {
            // We want the licenses from our current dot files
            let source_value = get_yaml_value(&entry.path().to_path_buf(), "source_license");
            let doc_value = get_yaml_value(&entry.path().to_path_buf(), "documentation_license");

            // Keep track of the license strings, avoiding duplicates
            if !source_licenses.contains(&source_value) {
                source_licenses.push(source_value);
            }
            if !doc_licenses.contains(&doc_value) {
                doc_licenses.push(doc_value);
            }
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
}

/*
 * Extracts a value from a JSON file based on a string key.
*/
fn get_json_value(json_file: &PathBuf, key: &str) -> String {
    let mut value = String::new();

    // If the file doesn't exist, we can't do anything
    if json_file.exists() {
        // Open the file for reading
        let mut file = fs::File::open(&json_file)
            .expect("Error opening JSON file.");

        // Attempt to read the contents of the file
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("ERROR: Unable to read the JSON file for this component");

        let lines = contents.split("\n");
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                let part: Vec<&str> = line.split(":").collect();
                value = part[1].replace("\"", "").replace(",", "").trim().to_string();
            }
        }
    }
    else {
        panic!("JSON file {} not found, cannot extract data from it.", json_file.display());
    }

    value
}

/*
 * Replaces the value corresponding to a key in a JSON file
*/
fn update_json_value(json_file: &PathBuf, key: &str, value: &str) {
    if json_file.exists() {
        println!("Attempting to update value for {} in {}.", json_file.display(), key);

        // Open the file for reading
        let mut file = fs::File::open(&json_file)
            .expect("Error opening JSON file.");

        // Attempt to read the contents of the component's .sr file
        let mut contents = String::new();
        let mut new_contents = String::new();
        file.read_to_string(&mut contents).expect("ERROR: Unable to read the JSON file for this component");

        let lines = contents.split("\n");
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                // Grab the original value
                let part: Vec<&str> = line.split(":").collect();
                let old_value = part[1].replace("\"", "").replace(",", "").trim().to_string();

                // Scope the change to matching line and replace the original line with the new one
                let new_line = line.replace(&old_value, &value);
                new_contents = contents.replace(line, &new_line);
            }
        }

        // Make sure there's a change to write
        if !new_contents.is_empty() {
            // Try to write the contents back to the file
            fs::write(json_file, new_contents)
                .expect("Could not write to JSON file.");
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
        let mut file = fs::File::open(&yaml_file)
            .expect("Error opening yaml file.");

        // Attempt to read the contents of the file
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("ERROR: Unable to read the yaml file for this component");

        let lines = contents.split("\n");
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains(&key) {
                let part: Vec<&str> = line.split(":").collect();
                value = String::from(part[1].replace(",", "").trim());
            }
        }
    }
    else {
        panic!("yaml file {} not found, cannot extract data from it.", yaml_file.display());
    }

    value
}

/*
 * Replaces the value corresponding to a key in a yaml file
*/
fn update_yaml_value(yaml_file: &PathBuf, key: &str, value: &str) {
    if yaml_file.exists() {
        // Open the file for reading
        let mut file = fs::File::open(&yaml_file)
            .expect("Error opening yaml file.");

        // Attempt to read the contents of the component's .sr file
        let mut contents = String::new();
        let mut new_contents = String::new();
        file.read_to_string(&mut contents).expect("ERROR: Unable to read the yaml file for this component");

        let lines = contents.split("\n");
        for line in lines {
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
            fs::write(yaml_file, new_contents)
                .expect("Could not write to yaml file.");
        }
    }
}

/*
 * Gets the parent directory of the current component
*/
fn get_parent_dir(target_dir: &Path) -> PathBuf {
    // Get the parent directory of this component's directory
    let parent_dir = target_dir.parent()
        .expect("Could not get the parent directory of the target component.");

    parent_dir.to_path_buf()
}

/*
 * Gets the line ending that's appropriate for the OS we are running on.
 */
fn get_newline() -> String {
    let info = os_info::get();

    if info.os_type() == os_info::Type::Windows {
        String::from("\r\n")
    }
    else {
        String::from("\n")
    }
}

/*
    * Figures out what depth the component is at.
    * 0 = A top level component is probably being created
    * 1 = A top level component with no parent
    * 2 = A sub-component at depth n
    */
pub fn get_level(target_dir: &Path) -> u8 {
    let level: u8;

    // Allows us to check if there is a .sr file in the current directory
    let current_file = target_dir.join(".sr");

    // Allows us to check if there is a .sr file in the parent directory
    let parent_file = target_dir.join(".sr");

    // If the parent directory contains a .sr file, we have a sub-component, if not we have a top level component
    if !parent_file.exists() && !current_file.exists() {
        level = 0;
    }
    else if !parent_file.exists() && current_file.exists() {
        level = 1;
    }
    else {
        level = 2;
    }

    level
}

pub mod git_sr;
pub mod npm_sr;
pub mod templates;


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    extern crate git2;
    extern crate uuid;
    use std::path::PathBuf;
    use std::io::prelude::*;

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
        let source_license = super::get_yaml_value(&test_dir.join("toplevel").join(".sr"), "source_license");
        assert_eq!(source_license, "Unlicense");

        // Change the source license from the sample directory
        super::update_yaml_value(&test_dir.join("toplevel").join(".sr"), "source_license", "NotASourceLicense");

        // Make sure the source license changed
        let source_license = super::get_yaml_value(&test_dir.join("toplevel").join(".sr"), "source_license");
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
        super::update_json_value(&test_dir.join("toplevel").join("package.json"), "name", "NotAName");

        // Make sure the component name changed in package.json
        let name = super::get_json_value(&test_dir.join("toplevel").join("package.json"), "name");
        assert_eq!(name, "NotAName");

        // Read a non-existent key from package.json
        let name = super::get_json_value(&test_dir.join("toplevel").join("package.json"), "not_a_key");
        assert_eq!(name, "");
    }

    /*
     * Tests whether or not the licenses are collected into the license field of package.json correctly.
     */
    #[test]
    fn test_license_amalgamation() {
        let temp_dir = env::temp_dir();

        // Set up our temporary project directory for testing
        let test_dir = set_up(&temp_dir, "toplevel");

        // Make sure the license field starts with something other than the string we are looking for
        super::update_json_value(&test_dir.join("toplevel").join("package.json"), "license", "NotALicense");

        super::amalgamate_licenses(&test_dir.join("toplevel"));

        // Make sure that all of the licenses were outlined correctly
        let license = super::get_json_value(&test_dir.join("toplevel").join("package.json"), "license");
        assert!(license.contains("Unlicense"));
        assert!(license.contains("CC0-1.0"));
        assert!(license.contains("NotASourceLicense"));
        assert!(license.contains("NotADocLicense"));
        assert!(license.contains("CC-BY-4.0"));
        assert!(license.contains("AND"));
        assert!(license.contains("("));
        assert!(license.contains(")"));
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
        let license_listing = super::list_all_licenses(&test_dir);

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
        globals.insert("source_license".into(), liquid::value::Value::scalar("NotASourceLicense"));
        globals.insert("doc_license".into(), liquid::value::Value::scalar("NotADocLicense"));

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
        globals.insert("license".into(), liquid::value::Value::scalar("(NotASourceLicense AND NotADocLicense)"));

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
        fs::create_dir(&temp_dir)
            .expect("Could not create temporary directory for test.");

        super::generate_dot_file(&temp_dir, "NotASourceLicense", "NotADocLicense");

        let mut file = fs::File::open(&temp_dir.join(".sr")).expect("Unable to open the sr file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the sr file");

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
        fs::create_dir(&temp_dir)
            .expect("Could not create temporary directory for test.");

        super::generate_gitignore(&temp_dir);

        let mut file = fs::File::open(&temp_dir.join(".gitignore")).expect("Unable to open the gitignore file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the gitignore file");

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
        fs::create_dir(&temp_dir)
            .expect("Could not create temporary directory for test.");

        super::generate_package_json(&temp_dir, "TopLevel", "NotASourceLicense");

        let mut file = fs::File::open(&temp_dir.join("package.json")).expect("Unable to open the package.json file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the package.json file");

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
        fs::create_dir(&temp_dir)
            .expect("Could not create temporary directory for test.");

        super::generate_bom(&temp_dir, "TopLevel");

        let mut file = fs::File::open(&temp_dir.join("bom_data.yaml")).expect("Unable to open the bom_data.yaml file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the package.json file");

        assert!(contents.contains("# Bill of Materials Data for TopLevel"));
    }

    #[test]
    fn test_generate_readme() {
        let temp_dir = env::temp_dir();
        let uuid_dir = uuid::Uuid::new_v4();
        let test_dir_name = format!("temp_{}", uuid_dir);
        let temp_dir = temp_dir.join(test_dir_name);

        // Create the temporary directory we are going to be working with
        fs::create_dir(&temp_dir)
            .expect("Could not create temporary directory for test.");

        super::generate_readme(&temp_dir, "TopLevel");

        let mut file = fs::File::open(&temp_dir.join("README.md")).expect("Unable to open the README.md file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the package.json file");

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

        for line in &output.stdout {
            println!("{}", line);
        }

        for line in &output.stderr {
            println!("{}", line);
        }

        // We should not have gotten an error
        assert_eq!(0, output.status);
        assert_eq!(0, output.status);

        assert!(output.stdout[1].contains("Dependencies were updated successfully."));
    }

    /*
     * Sets up a test directory for our use.
     */
    fn set_up(temp_dir: &PathBuf, dir_name: &str) -> PathBuf {
        // let url = format!("git://127.0.0.1/{}", dir_name);
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
}
