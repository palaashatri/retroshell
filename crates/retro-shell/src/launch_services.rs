use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AppBundle {
    pub bundle_id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub entrypoint: String,
    pub supported_types: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileAssociation {
    pub extension: String,
    pub default_app: String,
    pub user_override: Option<String>,
}

pub struct LaunchServices {
    pub bundles: HashMap<String, AppBundle>,
    pub associations: HashMap<String, FileAssociation>,
    pub search_paths: Vec<String>,
}

impl LaunchServices {
    pub fn new() -> Self {
        let mut services = Self {
            bundles: HashMap::new(),
            associations: HashMap::new(),
            search_paths: vec!["/Applications".into(), "/User/Applications".into()],
        };
        services.setup_default_associations();
        services
    }

    fn setup_default_associations(&mut self) {
        let defaults = vec![
            ("txt", "com.retro.textedit"),
            ("rtf", "com.retro.textedit"),
            ("md", "com.retro.textedit"),
            ("png", "com.retro.imageviewer"),
            ("jpg", "com.retro.imageviewer"),
            ("jpeg", "com.retro.imageviewer"),
            ("gif", "com.retro.imageviewer"),
            ("zip", "com.retro.archiveutility"),
            ("pdf", "com.retro.textedit"),
        ];
        for (ext, app) in defaults {
            self.associations.insert(ext.to_string(), FileAssociation {
                extension: ext.to_string(),
                default_app: app.to_string(),
                user_override: None,
            });
        }
    }

    pub fn register_bundle(&mut self, bundle: AppBundle) {
        self.bundles.insert(bundle.bundle_id.clone(), bundle);
    }

    pub fn scan_applications(&mut self) {
        // Scan /Applications and /User/Applications directories
        // For now, register built-in apps
        let builtins = vec![
            ("com.retro.finder", "Finder", "0.1.0", "/Applications/Finder.app"),
            ("com.retro.settings", "Settings", "0.1.0", "/Applications/Settings.app"),
            ("com.retro.textedit", "TextEdit", "0.1.0", "/Applications/TextEdit.app"),
            ("com.retro.terminal", "Terminal", "0.1.0", "/Applications/Terminal.app"),
        ];
        for (id, name, version, path) in builtins {
            self.register_bundle(AppBundle {
                bundle_id: id.to_string(),
                name: name.to_string(),
                version: version.to_string(),
                path: path.to_string(),
                entrypoint: "main".to_string(),
                supported_types: vec![],
                permissions: vec![],
            });
        }
    }

    pub fn launch_app(&self, bundle_id: &str) -> Option<&AppBundle> {
        self.bundles.get(bundle_id)
    }

    pub fn app_for_extension(&self, extension: &str) -> Option<&str> {
        self.associations.get(extension)
            .map(|a| a.user_override.as_ref().unwrap_or(&a.default_app))
            .map(|s| s.as_str())
    }

    pub fn bundle_for_id(&self, bundle_id: &str) -> Option<&AppBundle> {
        self.bundles.get(bundle_id)
    }
}
