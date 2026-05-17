#![allow(dead_code)]

//! Site config merger.
//!
//! Merge rules (deliberate simplifications of the spec):
//!   * `Option<T>` — overlay wins when `Some`.
//!   * `String` — overlay wins when non-empty.
//!   * `bool` — overlay wins when `true` (true takes precedence over false).
//!   * `Vec` — overlay items appended after base items.
//!   * `HashMap` — overlay keys override base keys; non-overlapping keys retained.
//!   * Nested structs — recursively merged field-by-field.

use std::collections::HashMap;

use super::models::*;

fn merge_opt<T>(base: Option<T>, overlay: Option<T>) -> Option<T> {
    overlay.or(base)
}

fn merge_string(base: String, overlay: String) -> String {
    if overlay.is_empty() { base } else { overlay }
}

fn merge_bool(base: bool, overlay: bool) -> bool {
    base || overlay
}

fn merge_vec<T: Clone>(base: Vec<T>, overlay: Vec<T>) -> Vec<T> {
    let mut out = base;
    out.extend(overlay);
    out
}

fn merge_map<K: std::hash::Hash + Eq, V>(base: HashMap<K, V>, overlay: HashMap<K, V>) -> HashMap<K, V> {
    let mut out = base;
    for (k, v) in overlay {
        out.insert(k, v);
    }
    out
}

fn merge_php_ini(base: PhpIniOverrides, overlay: PhpIniOverrides) -> PhpIniOverrides {
    PhpIniOverrides {
        memory_limit: merge_opt(base.memory_limit, overlay.memory_limit),
        upload_max_filesize: merge_opt(base.upload_max_filesize, overlay.upload_max_filesize),
        post_max_size: merge_opt(base.post_max_size, overlay.post_max_size),
        max_execution_time: merge_opt(base.max_execution_time, overlay.max_execution_time),
        max_input_vars: merge_opt(base.max_input_vars, overlay.max_input_vars),
        max_file_uploads: merge_opt(base.max_file_uploads, overlay.max_file_uploads),
        session_gc_maxlifetime: merge_opt(base.session_gc_maxlifetime, overlay.session_gc_maxlifetime),
        date_timezone: merge_opt(base.date_timezone, overlay.date_timezone),
        extra: merge_map(base.extra, overlay.extra),
    }
}

fn merge_php(base: PhpSiteConfig, overlay: PhpSiteConfig) -> PhpSiteConfig {
    PhpSiteConfig {
        version: merge_opt(base.version, overlay.version),
        ini_overrides: merge_php_ini(base.ini_overrides, overlay.ini_overrides),
        xdebug_enabled: merge_bool(base.xdebug_enabled, overlay.xdebug_enabled),
        // For xdebug_mode, prefer overlay if not the default.
        xdebug_mode: if overlay.xdebug_mode == XdebugMode::Off { base.xdebug_mode } else { overlay.xdebug_mode },
    }
}

fn merge_framework(base: FrameworkSiteConfig, overlay: FrameworkSiteConfig) -> FrameworkSiteConfig {
    FrameworkSiteConfig {
        framework_override: merge_opt(base.framework_override, overlay.framework_override),
        detected_version: merge_opt(base.detected_version, overlay.detected_version),
    }
}

fn merge_basic_auth(base: BasicAuthConfig, overlay: BasicAuthConfig) -> BasicAuthConfig {
    BasicAuthConfig {
        enabled: merge_bool(base.enabled, overlay.enabled),
        realm: merge_string(base.realm, overlay.realm),
        users: merge_vec(base.users, overlay.users),
    }
}

fn merge_server(base: HttpServerSiteConfig, overlay: HttpServerSiteConfig) -> HttpServerSiteConfig {
    HttpServerSiteConfig {
        // Server type: overlay wins if it differs from default.
        server_type: if overlay.server_type == HttpServerType::default() && base.server_type != HttpServerType::default() {
            base.server_type
        } else {
            overlay.server_type
        },
        custom_directives: merge_string(base.custom_directives, overlay.custom_directives),
        client_max_body_size: merge_opt(base.client_max_body_size, overlay.client_max_body_size),
        read_timeout: merge_opt(base.read_timeout, overlay.read_timeout),
        extra_headers: merge_map(base.extra_headers, overlay.extra_headers),
        basic_auth: merge_basic_auth(base.basic_auth, overlay.basic_auth),
        redirects: merge_vec(base.redirects, overlay.redirects),
    }
}

fn merge_wp(base: WordPressConfig, overlay: WordPressConfig) -> WordPressConfig {
    WordPressConfig {
        multisite_enabled: merge_bool(base.multisite_enabled, overlay.multisite_enabled),
        multisite_type: if overlay.multisite_type == MultisiteType::default() && base.multisite_type != MultisiteType::default() {
            base.multisite_type
        } else {
            overlay.multisite_type
        },
        domain_current_site: merge_opt(base.domain_current_site, overlay.domain_current_site),
        wp_debug: merge_bool(base.wp_debug, overlay.wp_debug),
        wp_debug_log: merge_bool(base.wp_debug_log, overlay.wp_debug_log),
        wp_debug_display: merge_bool(base.wp_debug_display, overlay.wp_debug_display),
        script_debug: merge_bool(base.script_debug, overlay.script_debug),
        savequeries: merge_bool(base.savequeries, overlay.savequeries),
        wp_home: merge_opt(base.wp_home, overlay.wp_home),
        wp_siteurl: merge_opt(base.wp_siteurl, overlay.wp_siteurl),
        table_prefix: merge_string(base.table_prefix, overlay.table_prefix),
        wp_cache: merge_opt(base.wp_cache, overlay.wp_cache),
        wp_memory_limit: merge_opt(base.wp_memory_limit, overlay.wp_memory_limit),
        wp_max_memory_limit: merge_opt(base.wp_max_memory_limit, overlay.wp_max_memory_limit),
        disallow_file_edit: merge_opt(base.disallow_file_edit, overlay.disallow_file_edit),
        disallow_file_mods: merge_opt(base.disallow_file_mods, overlay.disallow_file_mods),
        force_ssl_admin: merge_opt(base.force_ssl_admin, overlay.force_ssl_admin),
        extra_config: merge_string(base.extra_config, overlay.extra_config),
    }
}

fn merge_laravel(base: LaravelConfig, overlay: LaravelConfig) -> LaravelConfig {
    LaravelConfig {
        environment: merge_opt(base.environment, overlay.environment),
        octane_enabled: merge_bool(base.octane_enabled, overlay.octane_enabled),
        octane_server: if overlay.octane_server == OctaneServer::default() && base.octane_server != OctaneServer::default() {
            base.octane_server
        } else {
            overlay.octane_server
        },
        octane_host: merge_opt(base.octane_host, overlay.octane_host),
        octane_port: merge_opt(base.octane_port, overlay.octane_port),
        octane_workers: merge_opt(base.octane_workers, overlay.octane_workers),
        horizon_enabled: merge_bool(base.horizon_enabled, overlay.horizon_enabled),
        telescope_enabled: merge_bool(base.telescope_enabled, overlay.telescope_enabled),
        pulse_enabled: merge_bool(base.pulse_enabled, overlay.pulse_enabled),
        reverb_enabled: merge_bool(base.reverb_enabled, overlay.reverb_enabled),
        reverb_port: merge_opt(base.reverb_port, overlay.reverb_port),
    }
}

fn merge_database(base: DatabaseSiteConfig, overlay: DatabaseSiteConfig) -> DatabaseSiteConfig {
    DatabaseSiteConfig {
        engine: merge_opt(base.engine, overlay.engine),
        // backup_before_destroy defaults to true; if overlay explicitly false but base true, keep base?
        // Simplest rule: overlay wins.
        backup_before_destroy: overlay.backup_before_destroy && base.backup_before_destroy,
        backup_path: merge_opt(base.backup_path, overlay.backup_path),
    }
}

fn merge_dev(base: DevSiteConfig, overlay: DevSiteConfig) -> DevSiteConfig {
    DevSiteConfig {
        node_version: merge_opt(base.node_version, overlay.node_version),
        package_manager: if overlay.package_manager == NodePackageManager::default() && base.package_manager != NodePackageManager::default() {
            base.package_manager
        } else {
            overlay.package_manager
        },
        dev_server_port: merge_opt(base.dev_server_port, overlay.dev_server_port),
        dev_env: merge_map(base.dev_env, overlay.dev_env),
        start_command: merge_opt(base.start_command, overlay.start_command),
    }
}

// ── Phase 13 — framework-specific sub-config mergers ─────────────────

fn merge_craft(base: CraftConfig, overlay: CraftConfig) -> CraftConfig {
    CraftConfig {
        environment: merge_opt(base.environment, overlay.environment),
        license_key: merge_opt(base.license_key, overlay.license_key),
        db_driver: merge_opt(base.db_driver, overlay.db_driver),
        use_project_config: merge_bool(base.use_project_config, overlay.use_project_config),
    }
}

fn merge_concretecms(base: ConcreteCmsConfig, overlay: ConcreteCmsConfig) -> ConcreteCmsConfig {
    ConcreteCmsConfig {
        environment: merge_opt(base.environment, overlay.environment),
        cache_enabled: merge_bool(base.cache_enabled, overlay.cache_enabled),
        // pretty_urls default = true; AND-style merge keeps "false" sticky.
        pretty_urls: base.pretty_urls && overlay.pretty_urls,
    }
}

fn merge_drupal(base: DrupalConfig, overlay: DrupalConfig) -> DrupalConfig {
    DrupalConfig {
        environment: merge_opt(base.environment, overlay.environment),
        trusted_host_patterns: merge_vec(base.trusted_host_patterns, overlay.trusted_host_patterns),
        cache_bins: merge_vec(base.cache_bins, overlay.cache_bins),
    }
}

fn merge_joomla(base: JoomlaConfig, overlay: JoomlaConfig) -> JoomlaConfig {
    JoomlaConfig {
        error_reporting: merge_opt(base.error_reporting, overlay.error_reporting),
        sef_urls: merge_bool(base.sef_urls, overlay.sef_urls),
        debug: merge_bool(base.debug, overlay.debug),
        cache_enabled: merge_bool(base.cache_enabled, overlay.cache_enabled),
    }
}

fn merge_magento(base: MagentoConfig, overlay: MagentoConfig) -> MagentoConfig {
    MagentoConfig {
        mode: merge_opt(base.mode, overlay.mode),
        indexer_mode: merge_opt(base.indexer_mode, overlay.indexer_mode),
    }
}

fn merge_octobercms(base: OctoberCmsConfig, overlay: OctoberCmsConfig) -> OctoberCmsConfig {
    OctoberCmsConfig {
        debug_mode: merge_bool(base.debug_mode, overlay.debug_mode),
        backend_path: merge_opt(base.backend_path, overlay.backend_path),
    }
}

fn merge_statamic(base: StatamicConfig, overlay: StatamicConfig) -> StatamicConfig {
    StatamicConfig {
        // flat_file default = true; both must be true to stay true.
        flat_file: base.flat_file && overlay.flat_file,
        git_integration: merge_bool(base.git_integration, overlay.git_integration),
        api_enabled: merge_bool(base.api_enabled, overlay.api_enabled),
    }
}

fn merge_optional<T>(
    base: Option<T>,
    overlay: Option<T>,
    merger: impl FnOnce(T, T) -> T,
) -> Option<T> {
    match (base, overlay) {
        (Some(b), Some(o)) => Some(merger(b, o)),
        (None, Some(o)) => Some(o),
        (Some(b), None) => Some(b),
        (None, None) => None,
    }
}

/// Merge two configs. `overlay` wins on conflicts (typically the site-root file
/// overlays the centralized file).
pub fn merge(base: SiteConfig, overlay: SiteConfig) -> SiteConfig {
    SiteConfig {
        php: merge_php(base.php, overlay.php),
        framework: merge_framework(base.framework, overlay.framework),
        server: merge_server(base.server, overlay.server),
        wordpress: match (base.wordpress, overlay.wordpress) {
            (Some(b), Some(o)) => Some(merge_wp(b, o)),
            (None, Some(o)) => Some(o),
            (Some(b), None) => Some(b),
            (None, None) => None,
        },
        laravel: match (base.laravel, overlay.laravel) {
            (Some(b), Some(o)) => Some(merge_laravel(b, o)),
            (None, Some(o)) => Some(o),
            (Some(b), None) => Some(b),
            (None, None) => None,
        },
        database: merge_database(base.database, overlay.database),
        development: merge_dev(base.development, overlay.development),
        phpmyadmin: merge_phpmyadmin(base.phpmyadmin, overlay.phpmyadmin),
        // Phase 13 — overlay-wins-if-Some for each framework sub-config.
        craft:       merge_optional(base.craft,       overlay.craft,       merge_craft),
        concretecms: merge_optional(base.concretecms, overlay.concretecms, merge_concretecms),
        drupal:      merge_optional(base.drupal,      overlay.drupal,      merge_drupal),
        joomla:      merge_optional(base.joomla,      overlay.joomla,      merge_joomla),
        magento:     merge_optional(base.magento,     overlay.magento,     merge_magento),
        octobercms:  merge_optional(base.octobercms,  overlay.octobercms,  merge_octobercms),
        statamic:    merge_optional(base.statamic,    overlay.statamic,    merge_statamic),
    }
}

fn merge_phpmyadmin(base: crate::site_config::models::PhpMyAdminSiteConfig, overlay: crate::site_config::models::PhpMyAdminSiteConfig) -> crate::site_config::models::PhpMyAdminSiteConfig {
    crate::site_config::models::PhpMyAdminSiteConfig {
        enabled: base.enabled || overlay.enabled,
        access_mode: if overlay.access_mode != crate::site_config::models::PmaAccessMode::default() { overlay.access_mode } else { base.access_mode },
        db_scope: if overlay.db_scope != crate::site_config::models::PmaDbScope::default() { overlay.db_scope } else { base.db_scope },
        db_name_override: overlay.db_name_override.or(base.db_name_override),
        db_user_override: overlay.db_user_override.or(base.db_user_override),
        path_alias: if !overlay.path_alias.is_empty() && overlay.path_alias != "/phpmyadmin" { overlay.path_alias } else { base.path_alias },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_option_wins_when_some() {
        let mut base = SiteConfig::default();
        base.php.version = Some("8.1".into());
        let mut overlay = SiteConfig::default();
        overlay.php.version = Some("8.3".into());
        let m = merge(base, overlay);
        assert_eq!(m.php.version, Some("8.3".into()));
    }

    #[test]
    fn base_option_kept_when_overlay_none() {
        let mut base = SiteConfig::default();
        base.php.version = Some("8.1".into());
        let overlay = SiteConfig::default();
        let m = merge(base, overlay);
        assert_eq!(m.php.version, Some("8.1".into()));
    }

    #[test]
    fn bool_overlay_true_wins() {
        let mut base = SiteConfig::default();
        base.wordpress = Some(WordPressConfig::default());
        let mut overlay = SiteConfig::default();
        overlay.wordpress = Some(WordPressConfig {
            wp_debug: true,
            ..Default::default()
        });
        let m = merge(base, overlay);
        assert!(m.wordpress.unwrap().wp_debug);
    }

    #[test]
    fn vec_overlay_appended_to_base() {
        let mut base = SiteConfig::default();
        base.server.redirects.push(RedirectRule {
            from: "/old".into(),
            to: "/new".into(),
            code: 301,
            enabled: true,
        });
        let mut overlay = SiteConfig::default();
        overlay.server.redirects.push(RedirectRule {
            from: "/foo".into(),
            to: "/bar".into(),
            code: 302,
            enabled: true,
        });
        let m = merge(base, overlay);
        assert_eq!(m.server.redirects.len(), 2);
    }

    #[test]
    fn map_overlay_key_overrides_base_key() {
        let mut base = SiteConfig::default();
        base.server.extra_headers.insert("X-Foo".into(), "1".into());
        let mut overlay = SiteConfig::default();
        overlay.server.extra_headers.insert("X-Foo".into(), "2".into());
        overlay.server.extra_headers.insert("X-Bar".into(), "3".into());
        let m = merge(base, overlay);
        assert_eq!(m.server.extra_headers.get("X-Foo"), Some(&"2".to_string()));
        assert_eq!(m.server.extra_headers.get("X-Bar"), Some(&"3".to_string()));
    }

    #[test]
    fn wordpress_some_overrides_none() {
        let base = SiteConfig::default();
        let mut overlay = SiteConfig::default();
        overlay.wordpress = Some(WordPressConfig::default());
        let m = merge(base, overlay);
        assert!(m.wordpress.is_some());
    }

    #[test]
    fn empty_overlay_string_preserves_base() {
        let mut base = SiteConfig::default();
        base.server.custom_directives = "add_header X-A 1;".into();
        let overlay = SiteConfig::default();
        let m = merge(base, overlay);
        assert_eq!(m.server.custom_directives, "add_header X-A 1;");
    }

    // ── Phase 13 — framework sub-config merge ─────────────────────────

    #[test]
    fn craft_some_overrides_none() {
        let base = SiteConfig::default();
        let mut overlay = SiteConfig::default();
        overlay.craft = Some(CraftConfig {
            environment: Some("dev".into()),
            ..Default::default()
        });
        let m = merge(base, overlay);
        assert_eq!(m.craft.unwrap().environment, Some("dev".into()));
    }

    #[test]
    fn magento_overlay_mode_wins() {
        let mut base = SiteConfig::default();
        base.magento = Some(MagentoConfig {
            mode: Some("default".into()),
            ..Default::default()
        });
        let mut overlay = SiteConfig::default();
        overlay.magento = Some(MagentoConfig {
            mode: Some("developer".into()),
            ..Default::default()
        });
        let m = merge(base, overlay);
        assert_eq!(m.magento.unwrap().mode, Some("developer".into()));
    }

    #[test]
    fn drupal_vec_overlay_appended() {
        let mut base = SiteConfig::default();
        base.drupal = Some(DrupalConfig {
            trusted_host_patterns: vec!["^a$".into()],
            ..Default::default()
        });
        let mut overlay = SiteConfig::default();
        overlay.drupal = Some(DrupalConfig {
            trusted_host_patterns: vec!["^b$".into()],
            ..Default::default()
        });
        let m = merge(base, overlay);
        let v = m.drupal.unwrap().trusted_host_patterns;
        assert_eq!(v.len(), 2);
        assert!(v.contains(&"^a$".to_string()));
        assert!(v.contains(&"^b$".to_string()));
    }

    #[test]
    fn octobercms_debug_overlay_true_wins() {
        let mut base = SiteConfig::default();
        base.octobercms = Some(OctoberCmsConfig::default());
        let mut overlay = SiteConfig::default();
        overlay.octobercms = Some(OctoberCmsConfig {
            debug_mode: true,
            ..Default::default()
        });
        let m = merge(base, overlay);
        assert!(m.octobercms.unwrap().debug_mode);
    }

    #[test]
    fn statamic_base_kept_when_overlay_missing() {
        let mut base = SiteConfig::default();
        base.statamic = Some(StatamicConfig {
            flat_file: true,
            git_integration: true,
            api_enabled: true,
        });
        let overlay = SiteConfig::default();
        let m = merge(base, overlay);
        let s = m.statamic.unwrap();
        assert!(s.flat_file);
        assert!(s.git_integration);
        assert!(s.api_enabled);
    }

    #[test]
    fn joomla_bool_overlay_true_wins() {
        let mut base = SiteConfig::default();
        base.joomla = Some(JoomlaConfig::default());
        let mut overlay = SiteConfig::default();
        overlay.joomla = Some(JoomlaConfig {
            debug: true,
            sef_urls: true,
            ..Default::default()
        });
        let m = merge(base, overlay);
        let j = m.joomla.unwrap();
        assert!(j.debug);
        assert!(j.sef_urls);
    }

    #[test]
    fn concretecms_pretty_urls_false_is_sticky() {
        let mut base = SiteConfig::default();
        base.concretecms = Some(ConcreteCmsConfig { pretty_urls: false, ..Default::default() });
        let mut overlay = SiteConfig::default();
        overlay.concretecms = Some(ConcreteCmsConfig { pretty_urls: true, ..Default::default() });
        let m = merge(base, overlay);
        // AND-merge: false in either should keep false.
        assert!(!m.concretecms.unwrap().pretty_urls);
    }
}
