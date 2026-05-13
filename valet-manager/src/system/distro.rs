use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum DistroKind {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    Unknown,
}

pub fn parse_os_release() -> DistroKind {
    let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
    parse_id_from_content(&content)
}

pub(crate) fn parse_id_from_content(content: &str) -> DistroKind {
    for line in content.lines() {
        if let Some(id) = line.strip_prefix("ID=") {
            let id = id.trim_matches('"').to_lowercase();
            return match id.as_str() {
                "ubuntu" => DistroKind::Ubuntu,
                "debian" => DistroKind::Debian,
                "fedora" => DistroKind::Fedora,
                "arch" => DistroKind::Arch,
                _ => DistroKind::Unknown,
            };
        }
    }
    DistroKind::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ubuntu_unquoted() {
        assert_eq!(parse_id_from_content("ID=ubuntu\n"), DistroKind::Ubuntu);
    }

    #[test]
    fn parses_fedora_quoted() {
        assert_eq!(parse_id_from_content("ID=\"fedora\"\n"), DistroKind::Fedora);
    }

    #[test]
    fn parses_arch() {
        assert_eq!(parse_id_from_content("NAME=Arch Linux\nID=arch\n"), DistroKind::Arch);
    }

    #[test]
    fn returns_unknown_for_missing_id() {
        assert_eq!(parse_id_from_content("NAME=SomeOS\n"), DistroKind::Unknown);
    }

    #[test]
    fn returns_unknown_for_empty() {
        assert_eq!(parse_id_from_content(""), DistroKind::Unknown);
    }
}
