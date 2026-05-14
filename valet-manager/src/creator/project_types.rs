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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectGroup {
    Laravel,
    WordPress,
    Php,
    Node,
    Static,
}

#[derive(Debug, Clone)]
pub struct ProjectType {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub group: ProjectGroup,
    pub required_tools: Vec<CliTool>,
    pub options: Vec<ProjectOption>,
    pub duration_warning: Option<String>,
    pub show_manual_download: bool,
    pub post_install_note: Option<String>,
    pub default_port: Option<u16>,
}

/// Returns all project types (Laravel + WordPress + PHP frameworks + Node + Static).
#[allow(dead_code)]
pub fn all_project_types() -> Vec<ProjectType> {
    vec![
        // WordPress ecosystem
        wordpress_blank(),
        wordpress_bedrock(),
        wordpress_sage(),
        wordpress_woocommerce(),
        wordpress_multisite(),
        // Laravel ecosystem
        laravel_blank(),
        laravel_breeze(),
        laravel_jetstream(),
        laravel_api(),
        // PHP frameworks
        symfony_full(),
        symfony_micro(),
        cakephp(),
        concretecms(),
        contao(),
        craft(),
        drupal(),
        jigsaw(),
        joomla(),
        kirby(),
        magento(),
        octobercms(),
        sculpin(),
        slim(),
        zend_laminas(),
        expressionengine(),
        // Node.js frameworks
        nextjs(),
        nuxt(),
        react_vite(),
        vue_vite(),
        sveltekit(),
        astro(),
        // Static HTML
        static_html(),
    ]
}

// ── WordPress ───────────────────────────────────────────────────────────────

fn wordpress_blank() -> ProjectType {
    ProjectType {
        id: "wordpress-blank".to_string(),
        display_name: "WordPress".to_string(),
        description: "Standard WordPress install".to_string(),
        group: ProjectGroup::WordPress,
        required_tools: vec![CliTool::WpCli, CliTool::WpCliValetCommand],
        options: wordpress_common_options(),
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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

// ── Laravel ─────────────────────────────────────────────────────────────────

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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
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

// ── PHP framework helpers ──────────────────────────────────────────────────

fn php_name_option() -> ProjectOption {
    ProjectOption {
        key: "name".to_string(),
        label: "Project Name".to_string(),
        option_type: OptionType::Text,
        default_value: "".to_string(),
        required: true,
    }
}

fn php_directory_option() -> ProjectOption {
    ProjectOption {
        key: "directory".to_string(),
        label: "Install Directory".to_string(),
        option_type: OptionType::Dir,
        default_value: "".to_string(),
        required: true,
    }
}

fn php_version_option() -> ProjectOption {
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

/// Standard database connection options (5 fields).
fn db_options() -> Vec<ProjectOption> {
    vec![
        ProjectOption {
            key: "db_name".to_string(),
            label: "Database Name".to_string(),
            option_type: OptionType::Text,
            default_value: "{name}".to_string(),
            required: false,
        },
        ProjectOption {
            key: "db_user".to_string(),
            label: "Database User".to_string(),
            option_type: OptionType::Text,
            default_value: "root".to_string(),
            required: false,
        },
        ProjectOption {
            key: "db_pass".to_string(),
            label: "Database Password".to_string(),
            option_type: OptionType::Password,
            default_value: "".to_string(),
            required: false,
        },
        ProjectOption {
            key: "db_host".to_string(),
            label: "Database Host".to_string(),
            option_type: OptionType::Text,
            default_value: "127.0.0.1".to_string(),
            required: false,
        },
        ProjectOption {
            key: "db_prefix".to_string(),
            label: "Table Prefix".to_string(),
            option_type: OptionType::Text,
            default_value: "".to_string(),
            required: false,
        },
    ]
}

// ── PHP frameworks ──────────────────────────────────────────────────────────

fn symfony_full() -> ProjectType {
    ProjectType {
        id: "symfony-full".to_string(),
        display_name: "Symfony (full)".to_string(),
        description: "Full Symfony webapp skeleton".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn symfony_micro() -> ProjectType {
    ProjectType {
        id: "symfony-micro".to_string(),
        display_name: "Symfony (micro)".to_string(),
        description: "Minimal Symfony skeleton for APIs/microservices".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn cakephp() -> ProjectType {
    ProjectType {
        id: "cakephp".to_string(),
        display_name: "CakePHP".to_string(),
        description: "CakePHP rapid development framework".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            ProjectOption {
                key: "version".to_string(),
                label: "Version".to_string(),
                option_type: OptionType::Select(vec![
                    "5.*".to_string(),
                    "4.*".to_string(),
                ]),
                default_value: "5.*".to_string(),
                required: false,
            },
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn concretecms() -> ProjectType {
    let mut options = vec![php_name_option(), php_directory_option()];
    options.extend(db_options());
    options.push(php_version_option());
    ProjectType {
        id: "concretecms".to_string(),
        display_name: "ConcreteCMS".to_string(),
        description: "ConcreteCMS (formerly Concrete5)".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options,
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn contao() -> ProjectType {
    ProjectType {
        id: "contao".to_string(),
        display_name: "Contao".to_string(),
        description: "Contao open source CMS".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: Some(
            "Run Contao install tool at https://{domain}/contao/install".to_string(),
        ),
        default_port: None,
    }
}

fn craft() -> ProjectType {
    let mut options = vec![php_name_option(), php_directory_option()];
    options.extend(db_options());
    options.push(php_version_option());
    ProjectType {
        id: "craft".to_string(),
        display_name: "Craft CMS".to_string(),
        description: "Craft CMS — content-first CMS".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options,
        duration_warning: None,
        show_manual_download: false,
        post_install_note: Some("Run `php craft setup` to complete installation".to_string()),
        default_port: None,
    }
}

fn drupal() -> ProjectType {
    ProjectType {
        id: "drupal".to_string(),
        display_name: "Drupal".to_string(),
        description: "Drupal CMS".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: Some(
            "Visit https://{domain}/install.php to complete Drupal setup".to_string(),
        ),
        default_port: None,
    }
}

fn jigsaw() -> ProjectType {
    ProjectType {
        id: "jigsaw".to_string(),
        display_name: "Jigsaw".to_string(),
        description: "Static site generator by Tighten".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer, CliTool::Npm],
        options: vec![
            php_name_option(),
            php_directory_option(),
            ProjectOption {
                key: "starter".to_string(),
                label: "Starter template".to_string(),
                option_type: OptionType::Select(vec![
                    "blank".to_string(),
                    "blog".to_string(),
                    "docs".to_string(),
                ]),
                default_value: "blank".to_string(),
                required: false,
            },
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn joomla() -> ProjectType {
    ProjectType {
        id: "joomla".to_string(),
        display_name: "Joomla".to_string(),
        description: "Joomla CMS".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: Some(
            "Visit https://{domain}/installation/index.php to complete Joomla setup".to_string(),
        ),
        default_port: None,
    }
}

fn kirby() -> ProjectType {
    ProjectType {
        id: "kirby".to_string(),
        display_name: "Kirby".to_string(),
        description: "Kirby — file-based CMS".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            ProjectOption {
                key: "edition".to_string(),
                label: "Edition".to_string(),
                option_type: OptionType::Select(vec![
                    "starterkit".to_string(),
                    "plainkit".to_string(),
                ]),
                default_value: "starterkit".to_string(),
                required: false,
            },
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn magento() -> ProjectType {
    let mut options = vec![php_name_option(), php_directory_option()];
    options.extend(db_options());
    options.push(ProjectOption {
        key: "admin_email".to_string(),
        label: "Admin Email".to_string(),
        option_type: OptionType::Text,
        default_value: "admin@example.com".to_string(),
        required: false,
    });
    options.push(php_version_option());
    ProjectType {
        id: "magento".to_string(),
        display_name: "Magento 2".to_string(),
        description: "Magento 2 e-commerce platform".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options,
        duration_warning: Some("Installation takes 5–15 minutes".to_string()),
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn octobercms() -> ProjectType {
    let mut options = vec![php_name_option(), php_directory_option()];
    options.extend(db_options());
    options.push(php_version_option());
    ProjectType {
        id: "octobercms".to_string(),
        display_name: "OctoberCMS".to_string(),
        description: "OctoberCMS built on Laravel".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options,
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn sculpin() -> ProjectType {
    ProjectType {
        id: "sculpin".to_string(),
        display_name: "Sculpin".to_string(),
        description: "Static site generator for PHP developers".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn slim() -> ProjectType {
    ProjectType {
        id: "slim".to_string(),
        display_name: "Slim Framework".to_string(),
        description: "Slim — PHP micro-framework".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn zend_laminas() -> ProjectType {
    ProjectType {
        id: "zend-laminas".to_string(),
        display_name: "Laminas".to_string(),
        description: "Laminas Project (formerly Zend Framework)".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![CliTool::Composer],
        options: vec![
            php_name_option(),
            php_directory_option(),
            php_version_option(),
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

fn expressionengine() -> ProjectType {
    ProjectType {
        id: "expressionengine".to_string(),
        display_name: "ExpressionEngine".to_string(),
        description: "License-required CMS — manual download required".to_string(),
        group: ProjectGroup::Php,
        required_tools: vec![],
        options: vec![
            php_name_option(),
            php_directory_option(),
        ],
        duration_warning: None,
        show_manual_download: true,
        post_install_note: None,
        default_port: None,
    }
}

// ── Node.js frameworks ──────────────────────────────────────────────────────

fn node_type(
    id: &str,
    display_name: &str,
    description: &str,
    default_port: u16,
) -> ProjectType {
    ProjectType {
        id: id.to_string(),
        display_name: display_name.to_string(),
        description: description.to_string(),
        group: ProjectGroup::Node,
        required_tools: vec![CliTool::Node, CliTool::Npm],
        options: vec![
            ProjectOption {
                key: "name".to_string(),
                label: "Project Name".to_string(),
                option_type: OptionType::Text,
                default_value: "".to_string(),
                required: true,
            },
            ProjectOption {
                key: "directory".to_string(),
                label: "Install Directory".to_string(),
                option_type: OptionType::Dir,
                default_value: "".to_string(),
                required: true,
            },
            ProjectOption {
                key: "port".to_string(),
                label: "Dev Server Port".to_string(),
                option_type: OptionType::Text,
                default_value: default_port.to_string(),
                required: false,
            },
            ProjectOption {
                key: "with_systemd".to_string(),
                label: "Create systemd service".to_string(),
                option_type: OptionType::Toggle,
                default_value: "false".to_string(),
                required: false,
            },
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: Some(default_port),
    }
}

fn nextjs() -> ProjectType {
    node_type(
        "nextjs",
        "Next.js",
        "React framework with SSR & file routing",
        3000,
    )
}

fn nuxt() -> ProjectType {
    node_type("nuxt", "Nuxt 4", "Vue full-stack framework with SSR", 3000)
}

fn react_vite() -> ProjectType {
    node_type(
        "react-vite",
        "React + Vite",
        "React SPA powered by the Vite dev server",
        5173,
    )
}

fn vue_vite() -> ProjectType {
    node_type(
        "vue-vite",
        "Vue + Vite",
        "Vue 3 SPA powered by the Vite dev server",
        5173,
    )
}

fn sveltekit() -> ProjectType {
    node_type(
        "sveltekit",
        "SvelteKit",
        "Svelte full-stack framework with SSR",
        5173,
    )
}

fn astro() -> ProjectType {
    node_type(
        "astro",
        "Astro",
        "Content-focused static site framework with islands",
        4321,
    )
}

// ── Static HTML ─────────────────────────────────────────────────────────────

fn static_html() -> ProjectType {
    ProjectType {
        id: "static-html".to_string(),
        display_name: "Static HTML".to_string(),
        description: "Plain HTML/CSS/JS — no PHP".to_string(),
        group: ProjectGroup::Static,
        required_tools: vec![],
        options: vec![
            ProjectOption {
                key: "name".to_string(),
                label: "Project Name".to_string(),
                option_type: OptionType::Text,
                default_value: "".to_string(),
                required: true,
            },
            ProjectOption {
                key: "directory".to_string(),
                label: "Install Directory".to_string(),
                option_type: OptionType::Dir,
                default_value: "".to_string(),
                required: true,
            },
            ProjectOption {
                key: "template".to_string(),
                label: "Starter template".to_string(),
                option_type: OptionType::Select(vec![
                    "blank".to_string(),
                    "tailwind".to_string(),
                    "bootstrap".to_string(),
                ]),
                default_value: "blank".to_string(),
                required: false,
            },
        ],
        duration_warning: None,
        show_manual_download: false,
        post_install_note: None,
        default_port: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_project_types_returns_at_least_25_types() {
        let types = all_project_types();
        assert!(types.len() >= 25, "expected >=25 types, got {}", types.len());
    }

    #[test]
    fn wordpress_blank_has_14_options() {
        let types = all_project_types();
        let wp = types.iter().find(|t| t.id == "wordpress-blank").expect("wordpress-blank exists");
        assert_eq!(wp.options.len(), 14);
    }

    #[test]
    fn laravel_blank_required_tools_contains_composer() {
        let types = all_project_types();
        let la = types.iter().find(|t| t.id == "laravel-blank").expect("laravel-blank exists");
        assert!(la.required_tools.contains(&CliTool::Composer));
    }

    #[test]
    fn laravel_blank_has_no_starter_kit_option() {
        let types = all_project_types();
        let la = types.iter().find(|t| t.id == "laravel-blank").expect("laravel-blank exists");
        assert!(!la.options.iter().any(|o| o.key == "starter_kit"));
    }

    #[test]
    fn project_group_variants() {
        assert_ne!(ProjectGroup::Laravel, ProjectGroup::WordPress);
        assert_ne!(ProjectGroup::Php, ProjectGroup::Node);
        assert_ne!(ProjectGroup::Node, ProjectGroup::Static);
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
        let php_opt = wp.options.iter().find(|o| o.key == "php_version").expect("php_version exists");
        assert_eq!(php_opt.default_value, "8.3");
    }

    #[test]
    fn wordpress_blank_locale_has_8_options() {
        let wp = wordpress_blank();
        let locale_opt = wp.options.iter().find(|o| o.key == "locale").expect("locale exists");
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
        let sk_opt = la.options.iter().find(|o| o.key == "starter_kit").expect("starter_kit exists");
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
        let sk_opt = la.options.iter().find(|o| o.key == "starter_kit").expect("starter_kit exists");
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
        let wp = types.iter().find(|t| t.id == "wordpress-sage").expect("wordpress-sage exists");
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
        let git_opt = la.options.iter().find(|o| o.key == "with_git").expect("with_git exists");
        assert_eq!(git_opt.default_value, "true");
    }

    #[test]
    fn laravel_blank_pest_default_is_false() {
        let la = laravel_blank();
        let pest_opt = la.options.iter().find(|o| o.key == "with_pest").expect("with_pest exists");
        assert_eq!(pest_opt.default_value, "false");
    }

    #[test]
    fn wordpress_multisite_toggle_option() {
        let wp = wordpress_multisite();
        let ms_opt = wp.options.iter().find(|o| o.key == "multisite").expect("multisite exists");
        assert!(matches!(ms_opt.option_type, OptionType::Toggle));
        assert_eq!(ms_opt.default_value, "false");
    }

    // ── New tests for added groups/types ────────────────────────────────

    #[test]
    fn php_group_contains_symfony_full() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "symfony-full").expect("symfony-full exists");
        assert_eq!(pt.group, ProjectGroup::Php);
    }

    #[test]
    fn node_group_contains_nextjs() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "nextjs").expect("nextjs exists");
        assert_eq!(pt.group, ProjectGroup::Node);
    }

    #[test]
    fn static_group_contains_static_html() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "static-html").expect("static-html exists");
        assert_eq!(pt.group, ProjectGroup::Static);
    }

    #[test]
    fn expressionengine_show_manual_download_is_true() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "expressionengine").expect("expressionengine exists");
        assert!(pt.show_manual_download);
    }

    #[test]
    fn magento_has_duration_warning() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "magento").expect("magento exists");
        assert!(pt.duration_warning.is_some());
        assert!(pt.duration_warning.as_ref().expect("warning set").contains("5"));
    }

    #[test]
    fn drupal_has_post_install_note() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "drupal").expect("drupal exists");
        assert!(pt.post_install_note.is_some());
        assert!(pt.post_install_note.as_ref().expect("note set").contains("install.php"));
    }

    #[test]
    fn nextjs_default_port_is_3000() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "nextjs").expect("nextjs exists");
        assert_eq!(pt.default_port, Some(3000));
    }

    #[test]
    fn node_types_have_with_systemd_toggle() {
        let types = all_project_types();
        let node_types: Vec<_> = types.iter().filter(|t| t.group == ProjectGroup::Node).collect();
        assert!(!node_types.is_empty());
        for pt in node_types {
            let opt = pt.options.iter().find(|o| o.key == "with_systemd")
                .unwrap_or_else(|| panic!("{} missing with_systemd", pt.id));
            assert!(matches!(opt.option_type, OptionType::Toggle));
        }
    }

    #[test]
    fn static_html_template_has_three_choices() {
        let types = all_project_types();
        let pt = types.iter().find(|t| t.id == "static-html").expect("static-html exists");
        let tpl = pt.options.iter().find(|o| o.key == "template").expect("template exists");
        if let OptionType::Select(opts) = &tpl.option_type {
            assert_eq!(opts.len(), 3);
            assert!(opts.contains(&"blank".to_string()));
            assert!(opts.contains(&"tailwind".to_string()));
            assert!(opts.contains(&"bootstrap".to_string()));
        } else {
            panic!("Expected Select variant");
        }
    }
}
