pub mod command_palette;
pub mod components;
pub mod modals;
pub mod panels;
pub mod screens;
pub mod sidebar;
pub mod theme;
pub mod windows;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DetectedFramework {
    Laravel,
    WordPress,
    Bedrock,
    CakePHP,
    ConcreteCms,
    Contao,
    Craft,
    Drupal,
    ExpressionEngine,
    Jigsaw,
    Joomla,
    Katana,
    Kirby,
    Magento,
    OctoberCms,
    Sculpin,
    Slim,
    Statamic,
    StaticHtml,
    Symfony,
    Zend,
    Unknown,
}

impl DetectedFramework {
    /// Returns all 22 variants (21 frameworks + Unknown).
    #[allow(dead_code)]
    pub fn all_variants() -> [DetectedFramework; 22] {
        use DetectedFramework::*;
        [
            Laravel,
            WordPress,
            Bedrock,
            CakePHP,
            ConcreteCms,
            Contao,
            Craft,
            Drupal,
            ExpressionEngine,
            Jigsaw,
            Joomla,
            Katana,
            Kirby,
            Magento,
            OctoberCms,
            Sculpin,
            Slim,
            Statamic,
            StaticHtml,
            Symfony,
            Zend,
            Unknown,
        ]
    }
}
