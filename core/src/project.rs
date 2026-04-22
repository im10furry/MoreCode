use crate::task::ProjectInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Project mode determines the level of detail and thoroughness in task execution.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProjectMode {
    /// Fast mode - MVP only, minimal features, quick delivery
    Fast,
    /// Medium mode - Complete features, balanced quality and speed
    Medium,
    /// Full mode - Complete features with comprehensive testing and documentation
    Full,
    /// Custom mode - User-defined requirements
    Custom,
}

impl ProjectMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Full => "full",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fast" => Some(Self::Fast),
            "medium" => Some(Self::Medium),
            "full" => Some(Self::Full),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Project status
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProjectStatus {
    /// Project is active and being worked on
    Active,
    /// Project is paused
    Paused,
    /// Project is completed
    Completed,
    /// Project is in error state
    Error,
}

/// Struct representing a project with its metadata and state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier
    pub id: String,
    /// Project information
    pub info: ProjectInfo,
    /// Project root path
    pub root_path: PathBuf,
    /// Project mode
    pub mode: ProjectMode,
    /// Project status
    pub status: ProjectStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Associated task IDs
    pub task_ids: Vec<String>,
}

impl Project {
    /// Create a new project
    pub fn new(id: impl Into<String>, info: ProjectInfo, root_path: impl AsRef<Path>, mode: ProjectMode) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            info,
            root_path: root_path.as_ref().to_path_buf(),
            mode,
            status: ProjectStatus::Active,
            created_at: now,
            updated_at: now,
            task_ids: Vec::new(),
        }
    }

    /// Update project mode
    pub fn set_mode(&mut self, mode: ProjectMode) {
        self.mode = mode;
        self.updated_at = Utc::now();
    }

    /// Update project status
    pub fn set_status(&mut self, status: ProjectStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Add a task to the project
    pub fn add_task(&mut self, task_id: impl Into<String>) {
        let task_id = task_id.into();
        if !self.task_ids.contains(&task_id) {
            self.task_ids.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// Check if a file path belongs to this project
    pub fn contains_path(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        path.starts_with(&self.root_path)
    }

    /// Get relative path from project root
    pub fn relative_path(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        path.as_ref().strip_prefix(&self.root_path).map(|p| p.to_path_buf())
    }
}

/// Project manager for handling multiple projects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectManager {
    /// List of projects
    pub projects: Vec<Project>,
    /// Current active project index
    pub active_project_index: Option<usize>,
}

impl ProjectManager {
    /// Create a new project manager
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            active_project_index: None,
        }
    }

    /// Add a new project
    pub fn add_project(&mut self, project: Project) -> usize {
        self.projects.push(project);
        let index = self.projects.len() - 1;
        if self.active_project_index.is_none() {
            self.active_project_index = Some(index);
        }
        index
    }

    /// Get active project
    pub fn active_project(&self) -> Option<&Project> {
        self.active_project_index.and_then(|index| self.projects.get(index))
    }

    /// Get active project mutable
    pub fn active_project_mut(&mut self) -> Option<&mut Project> {
        self.active_project_index.and_then(|index| self.projects.get_mut(index))
    }

    /// Set active project by index
    pub fn set_active_project(&mut self, index: usize) -> bool {
        if index < self.projects.len() {
            self.active_project_index = Some(index);
            true
        } else {
            false
        }
    }

    /// Switch to next project
    pub fn next_project(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let next_index = match self.active_project_index {
            Some(index) if index == self.projects.len() - 1 => 0,
            Some(index) => index + 1,
            None => 0,
        };
        self.active_project_index = Some(next_index);
    }

    /// Switch to previous project
    pub fn previous_project(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let prev_index = match self.active_project_index {
            Some(index) if index == 0 => self.projects.len() - 1,
            Some(index) => index - 1,
            None => self.projects.len() - 1,
        };
        self.active_project_index = Some(prev_index);
    }

    /// Find project by path
    pub fn find_project_by_path(&self, path: impl AsRef<Path>) -> Option<(usize, &Project)> {
        self.projects
            .iter()
            .enumerate()
            .find(|(_, project)| project.contains_path(path.as_ref()))
    }

    /// Find project by ID
    pub fn find_project_by_id(&self, id: &str) -> Option<(usize, &Project)> {
        self.projects
            .iter()
            .enumerate()
            .find(|(_, project)| project.id == id)
    }

    /// Remove project by index
    pub fn remove_project(&mut self, index: usize) -> bool {
        if index < self.projects.len() {
            self.projects.remove(index);
            if let Some(active_index) = self.active_project_index {
                if active_index >= self.projects.len() {
                    self.active_project_index = if self.projects.is_empty() {
                        None
                    } else {
                        Some(self.projects.len() - 1)
                    };
                } else if active_index > index {
                    self.active_project_index = Some(active_index - 1);
                }
            }
            true
        } else {
            false
        }
    }

    /// Identify project for a given file path
    /// Returns the index of the existing project or creates a new one
    pub fn identify_project(&mut self, path: impl AsRef<Path>) -> usize {
        let path = path.as_ref();
        
        // Check if path already belongs to an existing project
        if let Some((index, _)) = self.find_project_by_path(path) {
            return index;
        }
        
        // Find project root by looking for project files
        if let Some(project_root) = self.find_project_root(path) {
            // Check if this root already has a project
            if let Some((index, _)) = self.projects
                .iter()
                .enumerate()
                .find(|(_, project)| project.root_path == project_root)
            {
                return index;
            }
            
            // Create new project
            let project_info = self.create_project_info(&project_root);
            let project = Project::new(
                crate::id::generate_id(),
                project_info,
                project_root,
                ProjectMode::Medium, // Default mode
            );
            self.add_project(project)
        } else {
            // No project root found, create a project with the file's parent directory
            let parent_dir = path.parent().unwrap_or_else(|| Path::new("."));
            let project_info = ProjectInfo {
                name: parent_dir.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().into_owned(),
                description: "Unknown project".to_string(),
                version: None,
                language: "Unknown".to_string(),
                framework: None,
                license: None,
                repository_url: None,
            };
            let project = Project::new(
                crate::id::generate_id(),
                project_info,
                parent_dir,
                ProjectMode::Medium,
            );
            self.add_project(project)
        }
    }

    /// Find project root by looking for project files
    fn find_project_root(&self, path: &Path) -> Option<PathBuf> {
        let project_files = [
            "package.json", // Node.js
            "Cargo.toml", // Rust
            "pyproject.toml", // Python
            "setup.py", // Python
            "go.mod", // Go
            "pom.xml", // Java
            "build.gradle", // Java
            "Gemfile", // Ruby
            "requirements.txt", // Python
            "tsconfig.json", // TypeScript
            "webpack.config.js", // Webpack
            "vite.config.js", // Vite
            "angular.json", // Angular
            "package-lock.json", // Node.js
            "yarn.lock", // Node.js
            "pnpm-lock.yaml", // Node.js
        ];
        
        let mut current = path;
        while let Some(parent) = current.parent() {
            for file in &project_files {
                if parent.join(file).exists() {
                    return Some(parent.to_path_buf());
                }
            }
            current = parent;
        }
        None
    }

    /// Create project info from project root
    fn create_project_info(&self, project_root: &Path) -> ProjectInfo {
        // Try to read project info from project files
        if let Ok(contents) = std::fs::read_to_string(project_root.join("package.json")) {
            if let Ok(package) = serde_json::from_str::<serde_json::Value>(&contents) {
                return ProjectInfo {
                    name: package.get("name").and_then(|v| v.as_str()).unwrap_or_else(|| project_root.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().as_ref()).to_string(),
                    description: package.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    version: package.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    language: "JavaScript".to_string(),
                    framework: package.get("dependencies").and_then(|deps| {
                        deps.as_object().and_then(|obj| {
                            if obj.contains_key("react") {
                                Some("React".to_string())
                            } else if obj.contains_key("vue") {
                                Some("Vue".to_string())
                            } else if obj.contains_key("angular") {
                                Some("Angular".to_string())
                            } else {
                                None
                            }
                        })
                    }),
                    license: package.get("license").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    repository_url: package.get("repository").and_then(|repo| {
                        repo.as_object().and_then(|obj| {
                            obj.get("url").and_then(|v| v.as_str()).map(|s| s.to_string())
                        })
                    }),
                };
            }
        } else if let Ok(contents) = std::fs::read_to_string(project_root.join("Cargo.toml")) {
            if let Ok(cargo) = toml::from_str::<toml::Value>(&contents) {
                return ProjectInfo {
                    name: cargo.get("package").and_then(|pkg| pkg.get("name")).and_then(|v| v.as_str()).unwrap_or_else(|| project_root.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().as_ref()).to_string(),
                    description: cargo.get("package").and_then(|pkg| pkg.get("description")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    version: cargo.get("package").and_then(|pkg| pkg.get("version")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                    language: "Rust".to_string(),
                    framework: None,
                    license: cargo.get("package").and_then(|pkg| pkg.get("license")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                    repository_url: cargo.get("package").and_then(|pkg| pkg.get("repository")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                };
            }
        }
        
        // Default project info
        ProjectInfo {
            name: project_root.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().into_owned(),
            description: "Unknown project".to_string(),
            version: None,
            language: "Unknown".to_string(),
            framework: None,
            license: None,
            repository_url: None,
        }
    }

    /// Process multiple files and group them by project
    pub fn process_files(&mut self, files: &[impl AsRef<Path>]) -> Vec<(usize, Vec<PathBuf>)> {
        let mut project_files = std::collections::HashMap::new();
        
        for file in files {
            let file_path = file.as_ref().to_path_buf();
            let project_index = self.identify_project(&file_path);
            project_files.entry(project_index).or_insert_with(Vec::new).push(file_path);
        }
        
        project_files.into_iter().collect()
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}