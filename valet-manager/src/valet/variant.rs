use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ValetVariant {
    ValetLinux,
    ValetOfficial,
    ValetLinuxPlus,
}

#[derive(Debug, Clone)]
pub struct ValetPaths {
    pub config_root: PathBuf,
    pub nginx_dir: PathBuf,
    pub sites_dir: PathBuf,
    pub drivers_dir: PathBuf,
    pub log_dir: PathBuf,
    pub config_json: PathBuf,
    pub ca_dir: PathBuf,
}
