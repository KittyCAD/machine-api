use std::{
    env, fs,
    io::{Read, Write},
    path,
    process::Command,
};

use anyhow::{anyhow, Result};

use crate::build_support::cmake_ext::CmakeExt;

/// Cereal is a dependency for orcaslicer.
const CEREAL_VERSION: &str = "v1.3.2";
const CEREAL_GIT_REPO: &str = "https://github.com/USCiLab/cereal.git";

/// Glew is a dependency for orcaslicer.
const GLEW_VERSION: &str = "glew-2.2.0";
const GLEW_GIT_REPO: &str = "https://github.com/nigels-com/glew.git";

/// This is the release of glfw that we are pinned on.
/// These come from: https://github.com/glfw/glfw/tags
const GLFW_VERSION: &str = "3.4";
const GLFW_GIT_REPO: &str = "https://github.com/glfw/glfw.git";

/// Orcaslicer.
const ORCASLICER_VERSION: &str = "v2.1.1";
const ORCASLICER_GIT_REPO: &str = "https://github.com/SoftFever/OrcaSlicer.git";

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub git_repo: String,
    pub build: crate::build_support::build::Build,
    pub source_dir: String,
    pub build_dir: String,
    pub install_dir: String,
    pub lib_dir: String,
    pub include_dir: String,
    pub link_libs: Vec<String>,
}

impl Dependency {
    /// Create a new dependency.
    fn new(name: &str, version: &str, git_repo: &str, build: &crate::build_support::build::Build) -> Self {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            git_repo: git_repo.to_string(),
            build: build.clone(),
            source_dir: build.dep_source_dir(name),
            build_dir: build.dep_build_dir(name),
            install_dir: build.dep_install_dir(name),
            lib_dir: build.dep_lib_dir(name),
            include_dir: build.dep_include_dir(name),
            // We will fill in the link libs after it's built.
            link_libs: Vec::new(),
        }
    }

    /// Clone the given dependency into it's build directory.
    fn git_clone_repo(&self) -> Result<()> {
        // Check if the repo is already cloned.
        if path::Path::new(&self.source_dir).exists() {
            if let Some("1") = option_env!("SKIP_DEPS") {
                return Ok(());
            }

            // Make sure we have the right branch (or tag).
            let mut git_fetch = Command::new("git");

            git_fetch.arg("fetch").arg("origin").current_dir(&self.source_dir);

            // We want this to fail in all normal work environments, so this
            // envvar can be handy when traveling or other special situations
            // where someone doesn't have Internet.
            if option_env!("NO_INTERNET").is_none() {
                crate::build_support::run_command(&mut git_fetch, "git fetch")?;
            }

            let mut git_checkout = Command::new("git");
            git_checkout
                .arg("checkout")
                .arg(&self.version)
                .current_dir(&self.source_dir);
            crate::build_support::run_command(&mut git_checkout, "git checkout")?;

            let mut git_pull = Command::new("git");
            git_pull
                .arg("pull")
                .arg("origin")
                .arg(&self.version)
                .current_dir(&self.source_dir);

            if option_env!("NO_INTERNET").is_none() {
                crate::build_support::run_command(&mut git_pull, "git pull")?;
            }
        } else {
            // We need to clone the repo, since the path doesn't exist.
            let mut git_clone = Command::new("git");

            git_clone
                .arg("clone")
                .arg(&format!("{}.git", self.git_repo.trim_end_matches(".git")))
                .arg("--recursive")
                .arg("--branch")
                .arg(&self.version)
                .arg(&self.source_dir);

            crate::build_support::run_command(&mut git_clone, "git clone")?;
        }

        Ok(())
    }

    /// Copy the given "local" dependency into it's build directory.
    fn copy_local_submodule_repo(&self) -> Result<()> {
        let full_path = path::Path::new(&self.source_dir)
            .join("..")
            .join("..")
            .join("..")
            .join(&self.git_repo);

        if path::Path::new(&self.source_dir).exists() {
            let mut git_fetch = Command::new("git");

            git_fetch
                .arg("submodule")
                .arg("update")
                .arg(".")
                .current_dir(&full_path);
            crate::build_support::run_command(&mut git_fetch, "git submodule update")?;
        } else {
            fs::create_dir_all(&self.source_dir)?;
        }

        //git_repo points to a local path
        let mut options = fs_extra::dir::CopyOptions::new(); //Initialize default values for CopyOptions
        options.overwrite = true;

        let copy_from_dir = path::Path::new(&full_path);
        let copy_to_dir = path::Path::new(&self.source_dir).join("..");
        let from_paths = vec![copy_from_dir];
        fs_extra::copy_items(&from_paths, copy_to_dir, &options)?;

        Ok(())
    }

    /// Create our build directory.
    fn create_build_directory(&self) -> Result<()> {
        fs::create_dir_all(&self.build_dir)?;

        Ok(())
    }

    fn build_version_file_path(&self) -> path::PathBuf {
        path::Path::new(&self.build_dir).join(".version.txt")
    }

    /// Write the version of the build we are building.
    fn write_build_version(&self) -> Result<()> {
        let mut version_file = fs::File::create(self.build_version_file_path())?;
        version_file.write_all(self.version.as_bytes())?;

        Ok(())
    }

    /// Read the version of the build we are building.
    fn read_build_version(&self) -> Result<String> {
        if !self.build_version_file_path().exists() {
            // Return an empty string.
            return Ok(String::new());
        }

        let mut file = fs::File::open(self.build_version_file_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    /// Check if we have already built the library.
    fn check_built(&self) -> Result<bool> {
        let lib = self.name.to_string();

        let lib_exists = path::Path::new(&self.lib_dir)
            .join(crate::build_support::static_lib_filename(&lib))
            .exists();

        if !lib_exists {
            // If we are on windows, try again with a lib prefix.
            if self.build.target.is_windows() {
                let lib_exists = path::Path::new(&self.lib_dir)
                    .join(crate::build_support::static_lib_filename(&format!("lib{lib}")))
                    .exists();

                if !lib_exists {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        // Check the source version is the same as our build version.
        let built_version = self.read_build_version()?;
        if built_version != self.version {
            return Ok(false);
        }

        Ok(true)
    }

    fn run_cmake_install(&self, extra_args: Vec<&'static str>) -> Result<()> {
        let mut cmake_install = Command::new("cmake");
        cmake_install
            .current_dir(&self.build_dir)
            .args(["--build", "."])
            .args(["--target", "install"])
            .args(["--config", cmake_profile()])
            .args(extra_args)
            .args([
                "--parallel",
                &env::var("NUM_JOBS").unwrap_or_else(|_| num_cpus::get().to_string()),
            ]);

        crate::build_support::run_command(&mut cmake_install, "cmake install")
    }

    fn set_link_libs(&mut self) -> Result<()> {
        self.link_libs = if self.build.target.is_windows() {
            // If windows, there is a suffix after the library name, find library name here.
            if path::Path::new(&self.lib_dir).exists() {
                let lib = fs::read_dir(&self.lib_dir)?
                    .map(|e| e.unwrap())
                    .find(|f| f.file_name().to_string_lossy().starts_with(&self.name));
                //.ok_or_else(|| anyhow!("Could not find {} library", self.name))?;
                if let Some(lib) = lib {
                    vec![lib
                        .file_name()
                        .to_str()
                        .ok_or_else(|| anyhow!("Could not convert file name to string"))?
                        .split('.')
                        .next()
                        .ok_or_else(|| anyhow!("No library name"))?
                        .to_owned()]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![self.name.to_string()]
        };

        Ok(())
    }
}

pub trait BuildDependency {
    fn new(build: &crate::build_support::build::Build) -> Box<Self>
    where
        Self: Sized;
    fn build(&mut self) -> Result<()>;

    fn build_with(&mut self, _extra: &Dependency) -> Result<()> {
        Ok(())
    }

    fn include_dir(&self) -> String;

    fn lib_dir(&self) -> String;

    fn link_libs(&self) -> Vec<String>;

    fn is_static(&self) -> bool;

    fn is_whole_archive(&self) -> bool;
}

/// Our cereal dependency.
pub struct Cereal(Dependency);

impl BuildDependency for Cereal {
    fn new(build: &crate::build_support::build::Build) -> Box<Self> {
        Box::new(Cereal(Dependency::new(
            "cereal",
            CEREAL_VERSION,
            CEREAL_GIT_REPO,
            build,
        )))
    }

    /// Build cereal.
    fn build(&mut self) -> Result<()> {
        // Let's ensure that we have the latest version of cereal locally.
        match self.0.git_clone_repo() {
            Ok(()) => (),
            Err(e) => {
                // We can ignore the error if we have unstaged changes since it doesn't
                // really matter.
                if !e.to_string().contains("You have unstaged changes.") {
                    anyhow::bail!(e);
                }
            }
        }

        self.0.create_build_directory()?;

        // Check if we have already built the library.
        let built = self.0.check_built()?;
        if built {
            return Ok(());
        }

        // Configure cereal.
        let mut cmake = Command::new("cmake");
        cmake
            .current_dir(&self.0.build_dir)
            .arg(&self.0.source_dir)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", cmake_profile()))
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.0.install_dir))
            .arg(format!("-DBUILD_SHARED_LIBS={}", "OFF"))
            .arg(format!("-DSKIP_PORTABILITY_TEST={}", "ON"))
            .arg(format!("-DSKIP_PERFORMANCE_COMPARISON={}", "ON"))
            .arg(format!("-DJUST_INSTALL_CEREAL={}", "ON"))
            .arg(format!("-DBUILD_SANDBOX={}", "OFF"))
            .arg(format!("-DBUILD_DOC={}", "OFF"));

        cmake.set_cmake_env(&self.0.build)?;

        crate::build_support::run_command(&mut cmake, "cmake")?;

        self.0.run_cmake_install(Vec::new())?;

        self.0.write_build_version()?;

        Ok(())
    }

    fn include_dir(&self) -> String {
        self.0.include_dir.to_string()
    }

    fn lib_dir(&self) -> String {
        self.0.lib_dir.to_string()
    }

    fn link_libs(&self) -> Vec<String> {
        self.0.link_libs.clone()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn is_whole_archive(&self) -> bool {
        false
    }
}

/// Our glew dependency.
pub struct Glew(Dependency);

impl BuildDependency for Glew {
    fn new(build: &crate::build_support::build::Build) -> Box<Self> {
        Box::new(Glew(Dependency::new("glew", GLEW_VERSION, GLEW_GIT_REPO, build)))
    }

    /// Build glew.
    fn build(&mut self) -> Result<()> {
        // Let's ensure that we have the latest version of glew locally.
        match self.0.git_clone_repo() {
            Ok(()) => (),
            Err(e) => {
                // We can ignore the error if we have unstaged changes since it doesn't
                // really matter.
                if !e.to_string().contains("You have unstaged changes.") {
                    anyhow::bail!(e);
                }
            }
        }

        self.0.create_build_directory()?;

        // Check if we have already built the library.
        let built = self.0.check_built()?;
        if built {
            return Ok(());
        }

        // Configure glew.
        let mut cmake = Command::new("cmake");
        cmake
            .current_dir(&self.0.build_dir)
            .arg(&self.0.source_dir)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", cmake_profile()))
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.0.install_dir))
            .arg(format!("-DBUILD_SHARED_LIBS={}", "OFF"));

        cmake.set_cmake_env(&self.0.build)?;

        crate::build_support::run_command(&mut cmake, "cmake")?;

        self.0.run_cmake_install(Vec::new())?;

        self.0.write_build_version()?;

        Ok(())
    }

    fn include_dir(&self) -> String {
        self.0.include_dir.to_string()
    }

    fn lib_dir(&self) -> String {
        self.0.lib_dir.to_string()
    }

    fn link_libs(&self) -> Vec<String> {
        self.0.link_libs.clone()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn is_whole_archive(&self) -> bool {
        false
    }
}

/// Our glfw dependency.
pub struct Glfw(Dependency);

impl BuildDependency for Glfw {
    fn new(build: &crate::build_support::build::Build) -> Box<Self> {
        Box::new(Glfw(Dependency::new("glfw3", GLFW_VERSION, GLFW_GIT_REPO, build)))
    }

    /// Build glfw.
    fn build(&mut self) -> Result<()> {
        // Let's ensure that we have the latest version of glfw locally.
        match self.0.git_clone_repo() {
            Ok(()) => (),
            Err(e) => {
                // We can ignore the error if we have unstaged changes since it doesn't
                // really matter.
                if !e.to_string().contains("You have unstaged changes.") {
                    anyhow::bail!(e);
                }
            }
        }

        self.0.create_build_directory()?;

        // Check if we have already built the library.
        let built = self.0.check_built()?;
        if built {
            self.0.set_link_libs()?;
            // If so, return early.
            return Ok(());
        }

        // Configure glfw.
        let mut cmake = Command::new("cmake");
        cmake
            .current_dir(&self.0.build_dir)
            .arg(&self.0.source_dir)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", cmake_profile()))
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.0.install_dir))
            .arg(format!("-DCMAKE_INSTALL_LIBDIR={}", "lib"))
            .arg(format!("-DGLFW_BUILD_DOCS={}", "OFF"))
            .arg(format!("-DBUILD_SHARED_LIBS={}", "OFF"));

        cmake.set_cmake_env(&self.0.build)?;

        crate::build_support::run_command(&mut cmake, "cmake")?;

        self.0.run_cmake_install(Vec::new())?;

        self.0.set_link_libs()?;

        self.0.write_build_version()?;

        Ok(())
    }

    fn include_dir(&self) -> String {
        self.0.include_dir.to_string()
    }

    fn lib_dir(&self) -> String {
        self.0.lib_dir.to_string()
    }

    fn link_libs(&self) -> Vec<String> {
        self.0.link_libs.clone()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn is_whole_archive(&self) -> bool {
        false
    }
}

/// Our orcaslicer dependency.
pub struct Orcaslicer(Dependency);

impl BuildDependency for Orcaslicer {
    fn new(build: &crate::build_support::build::Build) -> Box<Self> {
        Box::new(Orcaslicer(Dependency::new(
            "orcaslicer",
            ORCASLICER_VERSION,
            ORCASLICER_GIT_REPO,
            build,
        )))
    }

    /// Build orcaslicer.
    fn build(&mut self) -> Result<()> {
        // Let's ensure that we have the latest version of orcaslicer locally.
        self.0.git_clone_repo()?;

        self.0.create_build_directory()?;

        // Check if we have already built the library.
        let built = self.0.check_built()?;
        if built {
            self.0.set_link_libs()?;

            // Add the glew dependency.
            //self.link_libs().push("glewstatic".to_string());

            // If so, return early.
            return Ok(());
        }

        // Configure orcaslicer.
        let mut cmake = Command::new("cmake");
        cmake
            .current_dir(&self.0.build_dir)
            .arg(&self.0.source_dir)
            .arg(format!("-DCMAKE_BUILD_TYPE={}", "Release"))
            .arg(format!("-DCMAKE_INSTALL_PREFIX={}", self.0.install_dir))
            .arg(format!("-DCMAKE_INSTALL_LIBDIR={}", "lib"))
            .arg(format!("-DBUILD_SHARED_LIBS={}", "OFF"))
            .arg(format!(
                "-Dglfw3_DIR={}",
                path::Path::new(&self.0.source_dir)
                    .join("..")
                    .join("glfw3")
                    .join("build")
                    .join("install")
                    .join("lib")
                    .join("cmake")
                    .join("glfw3")
                    .to_str()
                    .unwrap()
            ))
            .arg(format!(
                "-Dcereal_DIR={}",
                path::Path::new(&self.0.source_dir)
                    .join("..")
                    .join("cereal")
                    .join("build")
                    .join("install")
                    .join("lib")
                    .join("cmake")
                    .join("cereal")
                    .to_str()
                    .unwrap()
            ))
            .arg(format!("-DSLIC3R_GUI={}", "0"));

        cmake.set_cmake_env(&self.0.build)?;

        crate::build_support::run_command(&mut cmake, "cmake")?;

        self.0.run_cmake_install(Vec::new())?;

        self.0.set_link_libs()?;

        // Add the glew dependency.
        self.link_libs().push("glewstatic".to_string());

        self.0.write_build_version()?;

        Ok(())
    }

    fn include_dir(&self) -> String {
        self.0.include_dir.to_string()
    }

    fn lib_dir(&self) -> String {
        self.0.lib_dir.to_string()
    }

    fn link_libs(&self) -> Vec<String> {
        self.0.link_libs.clone()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn is_whole_archive(&self) -> bool {
        false
    }
}

fn cmake_profile() -> &'static str {
    if let Ok(s) = std::env::var("PROFILE") {
        if s == "debug" {
            return "Debug";
        }
    }
    "Release"
}
