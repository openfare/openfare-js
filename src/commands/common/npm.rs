use anyhow::{format_err, Context, Result};
// use std::collections::HashSet;

/// Parse and clean package version string.
///
/// Returns a structure which details common errors.
// fn get_parsed_version(
//     version: &Option<&str>,
// ) -> openfare_lib::extension::common::VersionParseResult {
//     if let Some(version) = version.and_then(|v| Some(v.to_string())) {
//         if version != "" {
//             return Ok(version);
//         }
//     }
//     Err(openfare_lib::extension::common::VersionError::from_missing_version())
// }

// type JsonObject = serde_json::Map<String, serde_json::Value>;

// fn parse_dependencies(
//     package_entry: &serde_json::Value,
//     include_dev_dependencies: bool,
// ) -> Result<Vec<openfare_lib::extension::Dependency>> {
//     let mut unprocessed_dependencies_sections: std::collections::VecDeque<&JsonObject> =
//         std::collections::VecDeque::new();

//     if let Some(dependencies) = package_entry["dependencies"].as_object() {
//         unprocessed_dependencies_sections.push_back(dependencies);
//     }

//     let mut all_dependencies = HashSet::new();
//     while let Some(dependencies) = unprocessed_dependencies_sections.pop_front() {
//         for (package_name, entry) in dependencies {
//             if !include_dev_dependencies && entry["dev"].as_bool().unwrap_or_default() {
//                 continue;
//             }

//             let version_parse_result = get_parsed_version(&entry["version"].as_str());
//             all_dependencies.insert(openfare_lib::extension::Dependency {
//                 name: package_name.clone(),
//                 version: version_parse_result,
//             });

//             if let Some(sub_dependencies) = entry["dependencies"].as_object() {
//                 unprocessed_dependencies_sections.push_back(sub_dependencies);
//             }
//         }
//     }

//     let mut all_dependencies: Vec<_> = all_dependencies.into_iter().collect();
//     all_dependencies.sort();
//     Ok(all_dependencies)
// }

/// Parse node module packages, return package OpenFare locks.
pub fn get_node_modules_locks(
    node_modules_directory: &std::path::PathBuf,
) -> Result<
    std::collections::BTreeMap<openfare_lib::package::Package, Option<openfare_lib::lock::Lock>>,
> {
    let results = std::fs::read_dir(node_modules_directory)?
        .filter_map(|package_directory| {
            if let Ok(package_directory) = package_directory {
                if !package_directory.path().is_dir() {
                    None
                } else {
                    Some(package_directory.path())
                }
            } else {
                None
            }
        })
        .filter_map(|package_directory| {
            let package_json_path = package_directory.join("package.json");
            let package = match get_package(&package_json_path) {
                Ok(k) => k,
                Err(_) => return None,
            };
            let lock = match get_lock(&package_directory) {
                Ok(k) => k,
                Err(_) => {
                    return Some((package, None));
                }
            };
            Some((package, lock))
        })
        .collect();
    Ok(results)
}

pub fn get_lock(
    package_directory: &std::path::PathBuf,
) -> Result<Option<openfare_lib::lock::Lock>> {
    let openfare_json_path = package_directory.join("OPENFARE.lock");
    let lock = if openfare_json_path.is_file() {
        Some(parse_openfare_json(&openfare_json_path)?)
    } else {
        None
    };
    Ok(lock)
}

pub fn get_package(
    package_json_file: &std::path::PathBuf,
) -> Result<openfare_lib::package::Package> {
    let file = std::fs::File::open(package_json_file)?;
    let reader = std::io::BufReader::new(file);
    let json: serde_json::Value = serde_json::from_reader(reader).context(format!(
        "Failed to parse package.json: {}",
        package_json_file.display()
    ))?;
    let name = json["name"]
        .as_str()
        .ok_or(format_err!(
            "Failed to parse package name from project.json: {}",
            package_json_file.display()
        ))?
        .to_string();
    let version = json["version"]
        .as_str()
        .ok_or(format_err!(
            "Failed to parse package version from package.json: {}",
            package_json_file.display()
        ))?
        .to_string();
    Ok(openfare_lib::package::Package { name, version })
}

fn parse_openfare_json(path: &std::path::PathBuf) -> Result<openfare_lib::lock::Lock> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let lock: openfare_lib::lock::Lock = serde_json::from_reader(reader)
        .context(format!("Failed to parse OPENFARE.lock: {}", path.display()))?;
    Ok(lock)
}
