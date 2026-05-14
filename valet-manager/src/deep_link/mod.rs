#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeepLink {
    OpenSite(String),  // site=...
    SwitchPhp(String), // switch=...
    OpenPanel(String), // panel=php-versions etc.
    Noop,
}

/// Parse a `valet-manager://` URL into a DeepLink.
#[allow(dead_code)]
pub fn parse(uri: &str) -> DeepLink {
    let Some(rest) = uri.strip_prefix("valet-manager://") else {
        return DeepLink::Noop;
    };
    let (action, query) = match rest.find('?') {
        Some(i) => (&rest[..i], &rest[i + 1..]),
        None => (rest, ""),
    };
    let mut params = std::collections::HashMap::new();
    for kv in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = kv.split_once('=') {
            let dec = urlencoded_decode(v);
            params.insert(k.to_string(), dec);
        }
    }
    match action {
        "open" => params
            .get("site")
            .cloned()
            .map(DeepLink::OpenSite)
            .unwrap_or(DeepLink::Noop),
        "php" => params
            .get("switch")
            .cloned()
            .map(DeepLink::SwitchPhp)
            .unwrap_or(DeepLink::Noop),
        "panel" => params
            .get("name")
            .cloned()
            .map(DeepLink::OpenPanel)
            .unwrap_or(DeepLink::Noop),
        _ => DeepLink::Noop,
    }
}

fn urlencoded_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                out.push(byte as char);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Map a parsed DeepLink to an AppCommand. None means no action.
#[allow(dead_code)]
pub fn to_command(link: &DeepLink) -> Option<crate::commands::AppCommand> {
    use crate::commands::AppCommand;
    use crate::state::app_state::Panel;
    match link {
        DeepLink::OpenSite(domain) => Some(AppCommand::OpenSiteInBrowser(domain.clone())),
        DeepLink::SwitchPhp(v) => Some(AppCommand::SwitchGlobalPhp(v.clone())),
        DeepLink::OpenPanel(name) => {
            let panel = match name.as_str() {
                "dashboard" => Some(Panel::Dashboard),
                "php-versions" => Some(Panel::PhpVersions),
                "php-extensions" => Some(Panel::PhpExtensions),
                "sites" => Some(Panel::Sites),
                "history" => Some(Panel::History),
                _ => None,
            };
            panel.map(AppCommand::OpenPanel)
        }
        DeepLink::Noop => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_open_site() {
        assert_eq!(
            parse("valet-manager://open?site=myapp.test"),
            DeepLink::OpenSite("myapp.test".into())
        );
    }
    #[test]
    fn parses_php_switch() {
        assert_eq!(
            parse("valet-manager://php?switch=8.3"),
            DeepLink::SwitchPhp("8.3".into())
        );
    }
    #[test]
    fn parses_panel_open() {
        assert_eq!(
            parse("valet-manager://panel?name=sites"),
            DeepLink::OpenPanel("sites".into())
        );
    }
    #[test]
    fn unknown_returns_noop() {
        assert_eq!(parse("valet-manager://nope"), DeepLink::Noop);
        assert_eq!(parse("http://anywhere"), DeepLink::Noop);
    }
    #[test]
    fn urlencoded_decoded() {
        assert_eq!(
            parse("valet-manager://open?site=my%20app.test"),
            DeepLink::OpenSite("my app.test".into())
        );
    }
    #[test]
    fn to_command_maps_open_site() {
        let cmd = to_command(&DeepLink::OpenSite("a.test".into()));
        assert!(matches!(
            cmd,
            Some(crate::commands::AppCommand::OpenSiteInBrowser(_))
        ));
    }
    #[test]
    fn to_command_maps_known_panel() {
        let cmd = to_command(&DeepLink::OpenPanel("sites".into()));
        assert!(matches!(
            cmd,
            Some(crate::commands::AppCommand::OpenPanel(_))
        ));
    }
    #[test]
    fn to_command_unknown_panel_is_none() {
        assert!(to_command(&DeepLink::OpenPanel("nope".into())).is_none());
    }
    #[test]
    fn to_command_noop_returns_none() {
        assert!(to_command(&DeepLink::Noop).is_none());
    }
}
