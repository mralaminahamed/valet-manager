use crate::cli_tools::registry::CliTool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionType {
    Text,
    Password,
    Select(Vec<String>),  // list of options
    Toggle,
    Dir,  // directory picker
}

#[derive(Debug, Clone)]
pub struct ProjectOption {
    pub key: String,
    pub label: String,
    pub option_type: OptionType,
    pub default_value: String,   // empty string if no default
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectGroup {
    Laravel,
    WordPress,
}

#[derive(Debug, Clone)]
pub struct ProjectType {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub group: ProjectGroup,
    pub required_tools: Vec<CliTool>,
    pub options: Vec<ProjectOption>,
}

/// Returns all 9 project types (5 WordPress + 4 Laravel)
#[allow(dead_code)]
pub fn all_project_types() -> Vec<ProjectType> {
    vec![
        wordpress_blank(),
        wordpress_bedrock(),
        wordpress_sage(),
        wordpress_woocommerce(),
        wordpress_multisite(),
        laravel_blank(),
        laravel_breeze(),
        laravel_jetstream(),
        laravel_api(),
    ]
}

fn wordpress_blank() -> ProjectType {
    ProjectType {
        id: "wordpress-blank".to_string(),
        display_name: "WordPress".to_string(),
        description: "Standard WordPress install".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        options: wordpress_common_options(),
    }
}

fn wordpress_bedrock() -> ProjectType {
    ProjectType {
        id: "wordpress-bedrock".to_string(),
        display_name: "Bedrock".to_string(),
        description: "Modern WordPress stack with Composer".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand, CliTool::Composer],
        options: wordpress_common_options(),
    }
}

fn wordpress_sage() -> ProjectType {
    ProjectType {
        id: "wordpress-sage".to_string(),
        display_name: "Sage Theme".to_string(),
        description: "WordPress + Sage starter theme".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![
            CliTool::WpCli,
            CliTool::WpCliValetCommand,
            CliTool::Composer,
            CliTool::Npm,
        ],
        options: wordpress_common_options(),
    }
}

fn wordpress_woocommerce() -> ProjectType {
    ProjectType {
        id: "wordpress-woocommerce".to_string(),
        display_name: "WooCommerce".to_string(),
        description: "WordPress + WooCommerce".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        options: wordpress_common_options(),
    }
}

fn wordpress_multisite() -> ProjectType {
    ProjectType {
        id: "wordpress-multisite".to_string(),
        display_name: "Multisite".to_string(),
        description: "WordPress Multisite network".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        options: wordpress_common_options(),
    }
}

/// Common WordPress options (14 total, in order)
fn wordpress_common_options() -> Vec<ProjectOption> {
    vec![
        // 1. name
        ProjectOption {
            key: "name".to_string(),
            label: "Project Name".to_string(),
            option_type: OptionType::Text,
            default_value: "".to_string(),
            required: true,
        },
        // 2. directory
        ProjectOption {
            key: "directory".to_string(),
            label: "Install Directory".to_string(),
            option_type: OptionType::Dir,
            default_value: "".to_string(),
            required: true,
        },
        // 3. php_version
        ProjectOption {
            key: "php_version".to_string(),
            label: "PHP Version".to_string(),
            option_type: OptionType::Select(vec![
                "8.1".to_string(),
                "8.2".to_string(),
                "8.3".to_string(),
                "8.4".to_string(),
            ]),
            default_value: "8.3".to_string(),
            required: false,
        },
        // 4. db_name
        ProjectOption {
            key: "db_name".to_string(),
            label: "Database Name".to_string(),
            option_type: OptionType::Text,
            default_value: "{name}".to_string(),  // placeholder for name
            required: false,
        },
        // 5. db_user
        ProjectOption {
            key: "db_user".to_string(),
            label: "Database User".to_string(),
            option_type: OptionType::Text,
            default_value: "root".to_string(),
            required: false,
        },
        // 6. db_pass
        ProjectOption {
            key: "db_pass".to_string(),
            label: "Database Password".to_string(),
            option_type: OptionType::Password,
            default_value: "".to_string(),
            required: false,
        },
        // 7. db_host
        ProjectOption {
            key: "db_host".to_string(),
            label: "Database Host".to_string(),
            option_type: OptionType::Text,
            default_value: "127.0.0.1".to_string(),
            required: false,
        },
        // 8. db_prefix
        ProjectOption {
            key: "db_prefix".to_string(),
            label: "Table Prefix".to_string(),
            option_type: OptionType::Text,
            default_value: "wp_".to_string(),
            required: false,
        },
        // 9. site_title
        ProjectOption {
            key: "site_title".to_string(),
            label: "Site Title".to_string(),
            option_type: OptionType::Text,
            default_value: "".to_string(),
            required: false,
        },
        // 10. admin_user
        ProjectOption {
            key: "admin_user".to_string(),
            label: "Admin Username".to_string(),
            option_type: OptionType::Text,
            default_value: "admin".to_string(),
            required: false,
        },
        // 11. admin_pass
        ProjectOption {
            key: "admin_pass".to_string(),
            label: "Admin Password".to_string(),
            option_type: OptionType::Password,
            default_value: "".to_string(),
            required: false,
        },
        // 12. admin_email
        ProjectOption {
            key: "admin_email".to_string(),
            label: "Admin Email".to_string(),
            option_type: OptionType::Text,
            default_value: "".to_string(),
            required: false,
        },
        // 13. locale
        ProjectOption {
            key: "locale".to_string(),
            label: "Locale".to_string(),
            option_type: OptionType::Select(vec![
                "en_US".to_string(),
                "en_GB".to_string(),
                "fr_FR".to_string(),
                "de_DE".to_string(),
                "es_ES".to_string(),
                "pt_BR".to_string(),
                "ja".to_string(),
                "zh_CN".to_string(),
            ]),
            default_value: "en_US".to_string(),
            required: false,
        },
        // 14. multisite
        ProjectOption {
            key: "multisite".to_string(),
            label: "Enable Multisite".to_string(),
            option_type: OptionType::Toggle,
            default_value: "false".to_string(),
            required: false,
        },
    ]
}

fn laravel_blank() -> ProjectType {
    ProjectType {
        id: "laravel-blank".to_string(),
        display_name: "Laravel".to_string(),
        description: "Fresh Laravel application".to_string(),
        group: ProjectGroup::Laravel,
        required_tools: vec![CliTool::Composer],
        options: vec![
            laravel_name_option(),
            laravel_directory_option(),
            laravel_php_version_option(),
            laravel_with_pest_option(),
            laravel_with_git_option(),
        ],
    }
}

fn laravel_breeze() -> ProjectType {
    ProjectType {
        id: "laravel-breeze".to_string(),
        display_name: "Laravel + Breeze".to_string(),
        description: "Laravel with Breeze starter kit".to_string(),
        group: ProjectGroup::Laravel,
        required_tools: vec![CliTool::Composer, CliTool::Npm],
        options: vec![
            laravel_name_option(),
            laravel_directory_option(),
            laravel_php_version_option(),
            ProjectOption {
                key: "starter_kit".to_string(),
                label: "Starter Kit".to_string(),
                option_type: OptionType::Select(vec![
                    "breeze-blade".to_string(),
                    "breeze-react".to_string(),
                    "breeze-vue".to_string(),
                ]),
                default_value: "breeze-blade".to_string(),
                required: false,
            },
            laravel_with_pest_option(),
            laravel_with_git_option(),
        ],
    }
}

fn laravel_jetstream() -> ProjectType {
    ProjectType {
        id: "laravel-jetstream".to_string(),
        display_name: "Laravel + Jetstream".to_string(),
        description: "Laravel with Jetstream".to_string(),
        group: ProjectGroup::Laravel,
        required_tools: vec![CliTool::Composer, CliTool::Npm],
        options: vec![
            laravel_name_option(),
            laravel_directory_option(),
            laravel_php_version_option(),
            ProjectOption {
                key: "starter_kit".to_string(),
                label: "Starter Kit".to_string(),
                option_type: OptionType::Select(vec![
                    "jetstream-livewire".to_string(),
                    "jetstream-inertia".to_string(),
                ]),
                default_value: "jetstream-livewire".to_string(),
                required: false,
            },
            laravel_with_pest_option(),
            laravel_with_git_option(),
        ],
    }
}

fn laravel_api() -> ProjectType {
    ProjectType {
        id: "laravel-api".to_string(),
        display_name: "Laravel API".to_string(),
        description: "Laravel API-only project".to_string(),
        group: ProjectGroup::Laravel,
        required_tools: vec![CliTool::Composer],
        options: vec![
            laravel_name_option(),
            laravel_directory_option(),
            laravel_php_version_option(),
            laravel_with_pest_option(),
            laravel_with_git_option(),
        ],
    }
}

fn laravel_name_option() -> ProjectOption {
    ProjectOption {
        key: "name".to_string(),
        label: "Project Name".to_string(),
        option_type: OptionType::Text,
        default_value: "".to_string(),
        required: true,
    }
}

fn laravel_directory_option() -> ProjectOption {
    ProjectOption {
        key: "directory".to_string(),
        label: "Install Directory".to_string(),
        option_type: OptionType::Dir,
        default_value: "".to_string(),
        required: true,
    }
}

fn laravel_php_version_option() -> ProjectOption {
    ProjectOption {
        key: "php_version".to_string(),
        label: "PHP Version".to_string(),
        option_type: OptionType::Select(vec![
            "8.1".to_string(),
            "8.2".to_string(),
            "8.3".to_string(),
            "8.4".to_string(),
        ]),
        default_value: "8.3".to_string(),
        required: false,
    }
}

fn laravel_with_pest_option() -> ProjectOption {
    ProjectOption {
        key: "with_pest".to_string(),
        label: "Use Pest".to_string(),
        option_type: OptionType::Toggle,
        default_value: "false".to_string(),
        required: false,
    }
}

fn laravel_with_git_option() -> ProjectOption {
    ProjectOption {
        key: "with_git".to_string(),
        label: "Initialize Git".to_string(),
        option_type: OptionType::Toggle,
        default_value: "true".to_string(),
        required: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_project_types_returns_9_types() {
        let types = all_project_types();
        assert_eq!(types.len(), 9);
    }

    #[test]
    fn wordpress_blank_has_14_options() {
        let types = all_project_types();
        let wp = types.iter().find(|t| t.id == "wordpress-blank").unwrap();
        assert_eq!(wp.options.len(), 14);
    }

    #[test]
    fn laravel_blank_required_tools_contains_composer() {
        let types = all_project_types();
        let la = types.iter().find(|t| t.id == "laravel-blank").unwrap();
        assert!(la.required_tools.contains(&CliTool::Composer));
    }

    #[test]
    fn laravel_blank_has_no_starter_kit_option() {
        let types = all_project_types();
        let la = types.iter().find(|t| t.id == "laravel-blank").unwrap();
        assert!(!la.options.iter().any(|o| o.key == "starter_kit"));
    }

    #[test]
    fn project_group_variants() {
        assert_ne!(ProjectGroup::Laravel, ProjectGroup::WordPress);
    }

    #[test]
    fn wordpress_blank_has_correct_group() {
        let wp = wordpress_blank();
        assert_eq!(wp.group, ProjectGroup::WordPress);
    }

    #[test]
    fn laravel_blank_has_correct_group() {
        let la = laravel_blank();
        assert_eq!(la.group, ProjectGroup::Laravel);
    }

    #[test]
    fn wordpress_variants_count() {
        let types = all_project_types();
        let wp_count = types.iter().filter(|t| t.group == ProjectGroup::WordPress).count();
        assert_eq!(wp_count, 5);
    }

    #[test]
    fn laravel_variants_count() {
        let types = all_project_types();
        let la_count = types.iter().filter(|t| t.group == ProjectGroup::Laravel).count();
        assert_eq!(la_count, 4);
    }

    #[test]
    fn wordpress_blank_first_option_is_name() {
        let wp = wordpress_blank();
        assert_eq!(wp.options[0].key, "name");
        assert_eq!(wp.options[0].label, "Project Name");
        assert!(wp.options[0].required);
    }

    #[test]
    fn wordpress_blank_second_option_is_directory() {
        let wp = wordpress_blank();
        assert_eq!(wp.options[1].key, "directory");
        assert_eq!(wp.options[1].label, "Install Directory");
        assert!(wp.options[1].required);
        assert!(matches!(wp.options[1].option_type, OptionType::Dir));
    }

    #[test]
    fn wordpress_blank_php_version_defaults_to_8_3() {
        let wp = wordpress_blank();
        let php_opt = wp.options.iter().find(|o| o.key == "php_version").unwrap();
        assert_eq!(php_opt.default_value, "8.3");
    }

    #[test]
    fn wordpress_blank_locale_has_8_options() {
        let wp = wordpress_blank();
        let locale_opt = wp.options.iter().find(|o| o.key == "locale").unwrap();
        if let OptionType::Select(opts) = &locale_opt.option_type {
            assert_eq!(opts.len(), 8);
        } else {
            panic!("Expected Select variant");
        }
    }

    #[test]
    fn laravel_breeze_has_6_options() {
        let la = laravel_breeze();
        assert_eq!(la.options.len(), 6);
    }

    #[test]
    fn laravel_breeze_has_starter_kit_option() {
        let la = laravel_breeze();
        assert!(la.options.iter().any(|o| o.key == "starter_kit"));
    }

    #[test]
    fn laravel_breeze_starter_kit_options() {
        let la = laravel_breeze();
        let sk_opt = la.options.iter().find(|o| o.key == "starter_kit").unwrap();
        if let OptionType::Select(opts) = &sk_opt.option_type {
            assert_eq!(opts.len(), 3);
            assert!(opts.contains(&"breeze-blade".to_string()));
            assert!(opts.contains(&"breeze-react".to_string()));
            assert!(opts.contains(&"breeze-vue".to_string()));
        } else {
            panic!("Expected Select variant");
        }
    }

    #[test]
    fn laravel_jetstream_starter_kit_options() {
        let la = laravel_jetstream();
        let sk_opt = la.options.iter().find(|o| o.key == "starter_kit").unwrap();
        if let OptionType::Select(opts) = &sk_opt.option_type {
            assert_eq!(opts.len(), 2);
            assert!(opts.contains(&"jetstream-livewire".to_string()));
            assert!(opts.contains(&"jetstream-inertia".to_string()));
        } else {
            panic!("Expected Select variant");
        }
    }

    #[test]
    fn wordpress_sage_has_composer_and_npm() {
        let types = all_project_types();
        let wp = types.iter().find(|t| t.id == "wordpress-sage").unwrap();
        assert!(wp.required_tools.contains(&CliTool::Composer));
        assert!(wp.required_tools.contains(&CliTool::Npm));
    }

    #[test]
    fn laravel_api_does_not_have_starter_kit() {
        let la = laravel_api();
        assert!(!la.options.iter().any(|o| o.key == "starter_kit"));
        assert_eq!(la.options.len(), 5);
    }

    #[test]
    fn laravel_blank_git_default_is_true() {
        let la = laravel_blank();
        let git_opt = la.options.iter().find(|o| o.key == "with_git").unwrap();
        assert_eq!(git_opt.default_value, "true");
    }

    #[test]
    fn laravel_blank_pest_default_is_false() {
        let la = laravel_blank();
        let pest_opt = la.options.iter().find(|o| o.key == "with_pest").unwrap();
        assert_eq!(pest_opt.default_value, "false");
    }

    #[test]
    fn wordpress_multisite_toggle_option() {
        let wp = wordpress_multisite();
        let ms_opt = wp.options.iter().find(|o| o.key == "multisite").unwrap();
        assert!(matches!(ms_opt.option_type, OptionType::Toggle));
        assert_eq!(ms_opt.default_value, "false");
    }
}
