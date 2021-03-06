//! Manages the config files (default location ~/.config/up/).

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{ensure, Result};
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};

use crate::args::Args;

#[derive(Default, Debug)]
pub struct Config {
    pub up_toml_path: Option<PathBuf>,
    pub config_toml: ConfigToml,
}

// TODO(gib): Work out the data structure for the toml files.
// TODO(gib): Work out how to make that structure easily accessible for users.
// TODO(gib): Provide a way for users to easily validate their toml files.
/// Basic config, doesn't parse the full set of update scripts.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigToml {
    /// Link options.
    link: Option<LinkConfigToml>,
    /// Path to tasks directory (default up_dir/tasks).
    tasks_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkConfigToml {
    from_dir: Option<String>,
    to_dir: Option<String>,
    backup_dir: Option<String>,
}

impl Config {
    /// Build the `Config` struct by parsing the config toml files.
    pub fn from(args: &Args) -> Result<Self> {
        let mut config_toml = ConfigToml::default();

        let maybe_up_toml_path = Self::get_up_toml_path(&args.config)?;
        let up_toml_path = if maybe_up_toml_path.exists() {
            let read_result = fs::read(&maybe_up_toml_path);
            if let Ok(file_contents) = read_result {
                let config_str = String::from_utf8_lossy(&file_contents);
                debug!("config_str: {:?}", config_str);
                config_toml = toml::from_str::<ConfigToml>(&config_str)?;
                debug!("Config_toml: {:?}", config_toml);
            }
            Some(maybe_up_toml_path)
        } else {
            None
        };

        Ok(Self {
            up_toml_path,
            config_toml,
        })
    }

    /// Get the path to the up.toml file, given the args passed to the cli.
    /// If the `args_config_path` is `$XDG_CONFIG_HOME/up/up.toml` (the default) then
    /// we assume it is unset and check the other options. Order is:
    /// 1. `--config`
    /// 2. `$UP_CONFIG`
    /// 3. `$XDG_CONFIG_HOME/up/up.toml`
    /// 4. `~/.config/up/toml`
    fn get_up_toml_path(args_config_path: &str) -> Result<PathBuf> {
        debug!("args_config_file: {}", args_config_path);
        let mut config_path: PathBuf;
        if args_config_path == "$XDG_CONFIG_HOME/up/up.toml" {
            let up_config_env = env::var("UP_CONFIG");

            if let Ok(config_path) = up_config_env {
                let config_path = PathBuf::from(config_path);
                ensure!(
                    config_path.exists(),
                    "Config path specified in UP_CONFIG env var doesn't exist.\n  config_path: {:?}",
                    &config_path,
                );
                return Ok(config_path);
            }

            trace!("Checking default config paths.");

            let home_dir = env::var("HOME")?;

            config_path = env::var("XDG_CONFIG_HOME")
                .map_or_else(|_err| Path::new(&home_dir).join(".config"), PathBuf::from);

            config_path.push("up");

            config_path.push("up.toml");
        } else {
            config_path = PathBuf::from(args_config_path);
            ensure!(
                config_path.exists(),
                "Config path specified in -c/--config arg doesn't exist.\n  config_path: {:?}",
                &config_path,
            );
        }
        Ok(config_path)
    }
}

#[cfg(test)]
#[path = "../tests"]
mod toml_paths_tests {
    use super::Config;

    #[path = "common/mod.rs"]
    mod common;

    use std::env;

    /// Test possible options for the up.toml. All run in one file as they modify the
    /// shared test environment.
    #[test]
    fn get_toml_paths() {
        // Set up paths.
        let default_path = "$XDG_CONFIG_HOME/up/up.toml";
        let fake_home_1 = common::fixtures_dir().join("fake_home_dir_with_upconfig");
        let config_toml_1 = fake_home_1.join(".config/up/up.toml");
        let fake_home_2 = common::fixtures_dir().join("fake_home_dir_without_upconfig");

        // With all options set, we should pick the one passed as command-line arg.
        let args_config_path = env::current_exe().unwrap();
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = Config::get_up_toml_path(args_config_path.to_str().unwrap());
        assert_eq!(config_path.unwrap(), args_config_path);

        // If nothing is passed as an arg but UP_CONFIG exists, we should use it.
        env::set_var("UP_CONFIG", args_config_path.clone());
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = Config::get_up_toml_path(default_path);
        assert_eq!(config_path.unwrap(), args_config_path);
        env::remove_var("UP_CONFIG");

        // If nothing is passed as an arg, we should use the XDG_CONFIG_HOME/up/up.toml.
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = Config::get_up_toml_path(default_path);
        assert_eq!(config_path.unwrap(), config_toml_1);

        // If XDG_CONFIG_HOME is set we should use it.
        env::set_var("HOME", fake_home_1.clone());
        // Set XDG_CONFIG_HOME to a non-existent path.
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".badconfig"));
        let config_path = Config::get_up_toml_path(default_path);
        assert_eq!(config_path.unwrap(), fake_home_1.join(".badconfig/up/up.toml"));

        // If XDG_CONFIG_HOME is missing we should use ~/.config/up/up.toml.
        env::remove_var("XDG_CONFIG_HOME");
        let config_path = Config::get_up_toml_path(default_path);
        assert_eq!(config_path.unwrap(), config_toml_1);

        // If XDG_CONFIG_HOME is missing and ~/.config doesn't exist we should use
        // still use it.
        env::set_var("HOME", fake_home_2.clone());
        env::remove_var("XDG_CONFIG_HOME");
        let config_path = Config::get_up_toml_path(default_path);
        assert_eq!(config_path.unwrap(), fake_home_2.join(".config/up/up.toml"),);

        // If none of the options are present and there is no ~ we should error.
        env::remove_var("HOME");
        env::remove_var("XDG_CONFIG_HOME");
        // Default arg, i.e. not passed.
        let config_path = Config::get_up_toml_path(default_path);
        assert!(config_path.is_err(), "Config path: {:?}", config_path);
    }
}
