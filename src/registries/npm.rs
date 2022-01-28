use anyhow::{format_err, Context, Result};
use std::io::Read;
use strum::IntoEnumIterator;

pub const HOST_NAME: &'static str = "npmjs.com";

/// Package dependency file types.
#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum DependencyFileType {
    PackageLockJson,
    PackageJson,
}

impl DependencyFileType {
    /// Return file name associated with dependency type.
    pub fn file_name(&self) -> std::path::PathBuf {
        match self {
            Self::PackageJson => std::path::PathBuf::from("package.json"),
            Self::PackageLockJson => std::path::PathBuf::from("package-lock.json"),
        }
    }
}

/// Package dependency file type and file path.
#[derive(Debug, Clone)]
pub struct DependencyFile {
    pub r#type: DependencyFileType,
    pub path: std::path::PathBuf,
}

/// Returns a vector of identified package dependency definition files.
///
/// Walks up the directory tree directory tree until the first positive result is found.
pub fn identify_dependency_files(
    working_directory: &std::path::PathBuf,
) -> Option<Vec<DependencyFile>> {
    assert!(working_directory.is_absolute());
    let mut working_directory = working_directory.clone();

    loop {
        // If at least one target is found, assume package is present.
        let mut found_dependency_file = false;

        let mut dependency_files: Vec<DependencyFile> = Vec::new();
        for dependency_file_type in DependencyFileType::iter() {
            let target_absolute_path = working_directory.join(dependency_file_type.file_name());
            if target_absolute_path.is_file() {
                found_dependency_file = true;
                dependency_files.push(DependencyFile {
                    r#type: dependency_file_type,
                    path: target_absolute_path,
                })
            }
        }
        if found_dependency_file {
            return Some(dependency_files);
        }

        // No need to move further up the directory tree after this loop.
        if working_directory == std::path::PathBuf::from("/") {
            break;
        }

        // Move further up the directory tree.
        working_directory.pop();
    }
    None
}

pub struct NodeModulesDirectories {
    pub local: Option<std::path::PathBuf>,
    // pub _global: Option<std::path::PathBuf>,
}

/// Returns paths to the local and global node_modules directories.
pub fn identify_node_modules_directories(
    working_directory: &std::path::PathBuf,
) -> Result<NodeModulesDirectories> {
    Ok(NodeModulesDirectories {
        local: local_node_modules_directory(&working_directory),
        // global: global_node_modules_directory()?,
    })
}

/// Returns a path to the global node_modules directory.
// fn global_node_modules_directory() -> Result<Option<std::path::PathBuf>> {
//     let handle = std::process::Command::new("npm")
//         .args(vec!["list", "-g", "--long", "--json"])
//         .stdin(std::process::Stdio::null())
//         .stderr(std::process::Stdio::piped())
//         .stdout(std::process::Stdio::piped())
//         .output()?;

//     let stdout = String::from_utf8_lossy(&handle.stdout);
//     let result: serde_json::Value = serde_json::from_str(&stdout)?;

//     let path = match result["dependencies"]["npm"]["path"].as_str() {
//         Some(p) => p,
//         None => return Ok(None),
//     };
//     let path = std::path::PathBuf::from(path).join("node_modules");
//     Ok(Some(path))
// }

/// Returns a path to the local node_modules directory.
fn local_node_modules_directory(
    working_directory: &std::path::PathBuf,
) -> Option<std::path::PathBuf> {
    assert!(working_directory.is_absolute());
    let mut working_directory = working_directory.clone();

    loop {
        let target_absolute_path = working_directory.join("node_modules");
        if target_absolute_path.is_dir() {
            return Some(target_absolute_path);
        }

        // No need to move further up the directory tree after this loop.
        if working_directory == std::path::PathBuf::from("/") {
            break;
        }

        // Move further up the directory tree.
        working_directory.pop();
    }
    None
}

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

/// Given package name, return latest version.
pub fn get_latest_version(package_name: &str) -> Result<Option<String>> {
    let json = get_registry_entry_json(&package_name)?;
    let versions = json["versions"]
        .as_object()
        .ok_or(format_err!("Failed to find versions JSON section."))?;
    let latest_version = versions.keys().last();
    Ok(latest_version.cloned())
}

fn get_registry_entry_json(package_name: &str) -> Result<serde_json::Value> {
    let handlebars_registry = handlebars::Handlebars::new();
    let json_url = handlebars_registry.render_template(
        "https://registry.npmjs.com/{{package_name}}",
        &maplit::btreemap! {"package_name" => package_name},
    )?;

    let mut result = reqwest::blocking::get(&json_url.to_string())?;
    let mut body = String::new();
    result.read_to_string(&mut body)?;

    Ok(serde_json::from_str(&body).context(format!("JSON was not well-formatted:\n{}", body))?)
}

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
    let openfare_json_path = package_directory.join(openfare_lib::lock::FILE_NAME);
    let lock = if openfare_json_path.is_file() {
        Some(parse_lock_file(&openfare_json_path)?)
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

fn parse_lock_file(path: &std::path::PathBuf) -> Result<openfare_lib::lock::Lock> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let lock: openfare_lib::lock::Lock = serde_json::from_reader(reader)
        .context(format!("Failed to parse OPENFARE.lock: {}", path.display()))?;
    Ok(lock)
}
