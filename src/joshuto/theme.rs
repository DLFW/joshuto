extern crate toml;
extern crate xdg;

use std::fs;
use std::process;

use joshuto;
use joshuto::sort;

#[derive(Debug, Deserialize)]
pub struct JoshutoRawColorTheme {
    fg: Option<i16>,
    bg: Option<i16>,
    bold: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JoshutoRawTheme {
    default_: Option<JoshutoRawColorTheme>,
    directory: Option<JoshutoRawColorTheme>,
    selection: Option<JoshutoRawColorTheme>,
    executable: Option<JoshutoRawColorTheme>,
    socket: Option<JoshutoRawColorTheme>,
    extensions: Option<HashMap<String, JoshutoRawColorTheme>>,
}

impl JoshutoRawTheme {
    #[allow(dead_code)]
    pub fn new() -> Self
    {
        JoshutoRawTheme {
            default_: None,
            directory: None,
            selection: None,
            executable: None,
            socket: None,
            extensions: None,
        }
    }

    pub fn flatten(self) -> JoshutoConfig
    {
        let username : String = whoami::username();
        let hostname : String = whoami::hostname();

        let column_ratio = match self.column_ratio {
            Some(s) => (s[0], s[1], s[2]),
            None => (1, 3, 4),
            };

        let scroll_offset: usize = self.scroll_offset.unwrap_or(6);

        let show_hidden: bool = self.show_hidden.unwrap_or(false);
        let sort_case_sensitive: bool = self.sort_case_sensitive.unwrap_or(false);
        let sort_reverse: bool = self.sort_reverse.unwrap_or(false);
        let sort_directories_first: bool = self.sort_directories_first.unwrap_or(true);

        let sort_struct = sort::SortStruct {
                show_hidden,
                sort_directories_first,
                sort_case_sensitive,
                sort_reverse,
            };

        let sort_type: sort::SortType = match self.sort_type {
            Some(s) => {
                match s.as_str() {
                    "natural" => sort::SortType::SortNatural(sort_struct),
                    "mtime" => sort::SortType::SortMtime(sort_struct),
                    _ => sort::SortType::SortNatural(sort_struct),
                }
            }
            _ => sort::SortType::SortNatural(sort_struct),
            };

        JoshutoConfig {
            username,
            hostname,
            scroll_offset,
            sort_type,
            column_ratio,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JoshutoConfig {
    pub username: String,
    pub hostname: String,
    pub scroll_offset: usize,
    pub sort_type: joshuto::sort::SortType,
    pub column_ratio: (usize, usize, usize),
}

impl JoshutoConfig {

    pub fn new() -> Self
    {
        let sort_struct = sort::SortStruct {
                show_hidden: false,
                sort_directories_first: true,
                sort_case_sensitive: false,
                sort_reverse: false,
            };
        let sort_type = sort::SortType::SortNatural(sort_struct);

        let username : String = whoami::username();
        let hostname : String = whoami::hostname();

        JoshutoConfig {
            username,
            hostname,
            scroll_offset: 6,
            sort_type,
            column_ratio: (1, 3, 4),
        }
    }

    fn read_config() -> Option<JoshutoRawConfig>
    {
        match xdg::BaseDirectories::with_profile(::PROGRAM_NAME, "") {
            Ok(dirs) => {
                let config_path = dirs.find_config_file(::CONFIG_FILE)?;
                match fs::read_to_string(&config_path) {
                    Ok(config_contents) => {
                        match toml::from_str(&config_contents) {
                            Ok(config) => {
                                Some(config)
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                                process::exit(1);
                            },
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                        None
                    },
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                None
            },
        }
    }

    pub fn get_config() -> Self
    {
        match JoshutoConfig::read_config() {
            Some(config) => {
                config.flatten()
            }
            None => {
                JoshutoConfig::new()
            }
        }
    }
}