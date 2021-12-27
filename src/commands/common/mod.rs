use anyhow::{format_err, Context, Result};
use std::io::Read;
use strum::IntoEnumIterator;
pub mod npm;

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
