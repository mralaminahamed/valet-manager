use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PhpVersion {
    pub version: String,
    pub full_version: String,
    pub binary_path: PathBuf,
    pub fpm_service: String,
    pub cli_ini_path: PathBuf,
    pub fpm_ini_path: PathBuf,
    pub conf_d_path: PathBuf,
    pub is_active: bool,
    pub fpm_running: bool,
}
