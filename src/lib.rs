use anyhow::Result;

mod commands;

#[derive(Clone, Debug)]
pub struct JsExtension {
    name_: String,
    registry_host_names_: Vec<String>,
    root_url_: url::Url,
    registry_human_url_template_: String,
}

impl openfare_lib::extension::FromLib for JsExtension {
    fn new() -> Self {
        Self {
            name_: "js".to_string(),
            registry_host_names_: vec!["npmjs.com".to_owned()],
            root_url_: url::Url::parse("https://www.npmjs.com").unwrap(),
            registry_human_url_template_:
                "https://www.npmjs.com/package/{{package_name}}/v/{{package_version}}".to_string(),
        }
    }
}

impl openfare_lib::extension::Extension for JsExtension {
    fn name(&self) -> String {
        self.name_.clone()
    }

    fn registries(&self) -> Vec<String> {
        self.registry_host_names_.clone()
    }

    fn package_dependencies_configs(
        &self,
        package_name: &str,
        package_version: &Option<&str>,
        extension_args: &Vec<String>,
    ) -> Result<
        openfare_lib::extension::commands::package_dependencies_configs::PackageDependenciesConfigs,
    > {
        commands::package_dependencies_configs(
            &self,
            &package_name,
            &package_version,
            &extension_args,
        )
    }

    fn fs_defined_dependencies_configs(
        &self,
        working_directory: &std::path::PathBuf,
        extension_args: &Vec<String>,
    ) -> Result<openfare_lib::extension::commands::fs_defined_dependencies_configs::FsDefinedDependenciesConfigs>{
        commands::fs_defined_dependencies_configs(&working_directory, &extension_args)
    }
}
