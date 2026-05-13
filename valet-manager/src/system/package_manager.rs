use crate::system::distro::{parse_os_release, DistroKind};

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
}

pub fn detect_package_manager() -> PackageManager {
    pm_for_distro(&parse_os_release())
}

pub fn pm_for_distro(kind: &DistroKind) -> PackageManager {
    match kind {
        DistroKind::Ubuntu | DistroKind::Debian => PackageManager::Apt,
        DistroKind::Fedora => PackageManager::Dnf,
        DistroKind::Arch => PackageManager::Pacman,
        DistroKind::Unknown => PackageManager::Apt,
    }
}

pub async fn list_installed_php_packages(pm: &PackageManager) -> Vec<String> {
    let (cmd, arg): (&str, &str) = match pm {
        PackageManager::Apt    => ("bash", "apt list --installed 2>/dev/null | grep php"),
        PackageManager::Dnf    => ("bash", "dnf list installed 2>/dev/null | grep php"),
        PackageManager::Pacman => ("bash", "pacman -Q 2>/dev/null | grep php"),
    };

    let Ok(output) = tokio::process::Command::new(cmd)
        .args(["-c", arg])
        .output()
        .await
    else {
        return Vec::new();
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.split('/').next().unwrap_or(l).trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubuntu_maps_to_apt() {
        assert_eq!(pm_for_distro(&DistroKind::Ubuntu), PackageManager::Apt);
    }

    #[test]
    fn debian_maps_to_apt() {
        assert_eq!(pm_for_distro(&DistroKind::Debian), PackageManager::Apt);
    }

    #[test]
    fn fedora_maps_to_dnf() {
        assert_eq!(pm_for_distro(&DistroKind::Fedora), PackageManager::Dnf);
    }

    #[test]
    fn arch_maps_to_pacman() {
        assert_eq!(pm_for_distro(&DistroKind::Arch), PackageManager::Pacman);
    }

    #[test]
    fn unknown_falls_back_to_apt() {
        assert_eq!(pm_for_distro(&DistroKind::Unknown), PackageManager::Apt);
    }
}
