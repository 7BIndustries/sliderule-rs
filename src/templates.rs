extern crate os_info;

/*
 * Returns the Liquid template for the bom_data.yaml file
 */
pub fn bom_data_yaml_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("# Bill of Materials Data for {{name}}");
    contents.push_str(nl);
    contents.push_str("parts:");
    contents.push_str(nl);
    contents.push_str("  component_1:");
    contents.push_str(nl);
    contents.push_str("    options:");
    contents.push_str(nl);
    contents.push_str("    - specific_component_variation");
    contents.push_str(nl);
    contents.push_str("    default_option: 0");
    contents.push_str(nl);
    contents.push_str("    quantity: 1");
    contents.push_str(nl);
    contents.push_str("    quantity_units: part");
    contents.push_str(nl);
    contents.push_str("    name: Sample Component");
    contents.push_str(nl);
    contents.push_str("    notes: ''");
    contents.push_str(nl);
    contents.push_str(nl);
    contents.push_str("order:");
    contents.push_str(nl);
    contents.push_str("  -component_1");
    contents.push_str(nl);

    contents
}

/*
 * Returns the Liquid template for the .gitignore file
 */
pub fn gitignore_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("# Dependency directories");
    contents.push_str(nl);
    contents.push_str("node_modules/");
    contents.push_str(nl);
    contents.push_str(nl);
    contents.push_str("# Distribution directory");
    contents.push_str(nl);
    contents.push_str("dist/");
    contents.push_str(nl);

    contents
}

/*
 * Returns the Liquid template for the package.json file
 */
pub fn package_json_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("{");
    contents.push_str(nl);
    contents.push_str("  \"name\": \"{{name}}\",");
    contents.push_str(nl);
    contents.push_str("  \"version\": \"1.0.0\",");
    contents.push_str(nl);
    contents.push_str("  \"description\": \"Sliderule DOF component.\",");
    contents.push_str(nl);
    contents.push_str("  \"license\": \"{{license}}\",");
    contents.push_str(nl);
    contents.push_str("  \"dependencies\": {");
    contents.push_str(nl);
    contents.push_str("  }");
    contents.push_str(nl);
    contents.push_str("}");
    contents.push_str(nl);

    contents
}

/*
 * Returns the Liquid template for the readme file
 */
pub fn readme_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("# {{name}}");
    contents.push_str(nl);
    contents.push_str("New Sliderule component.");
    contents.push_str(nl);
    contents.push_str(nl);
    contents.push_str("---");
    contents.push_str(nl);
    contents.push_str("Developed in [Sliderule](http://sliderule.io) an implementation of the [Distributed OSHW Framework](http://dof.sliderule.io).");
    contents.push_str(nl);

    contents
}

/*
 * Returns the Liquid template text for the .sr file
 */
pub fn sr_file_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("source_license: {{source_license}},");
    contents.push_str(nl);
    contents.push_str("documentation_license: {{doc_license}}");
    contents.push_str(nl);

    contents
}

/*
 * Returns the Liquid template text for a part item entry in parts.yaml or tools.yaml
 */
pub fn item_template() -> String {
    let nl = &get_newline();

    let mut contents = String::from("{{item_name}}:");
    contents.push_str(nl);
    contents.push_str("  id: {{item_name}}");
    contents.push_str(nl);
    contents.push_str("  description: {{item_description}}");
    contents.push_str(nl);
    contents.push_str("  quantity: {{item_qty}}");
    contents.push_str(nl);
    contents.push_str("  quantityUnits: {{quantity_units}}");
    contents.push_str(nl);
    contents.push_str("  options:");
    contents.push_str(nl);
    contents.push_str("  - {{component_name}}");
    contents.push_str(nl);
    contents.push_str("  selectedOption: {{component_name}}");
    contents.push_str(nl);
    contents.push_str("  notes: {{item_notes}}");

    return contents;
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
