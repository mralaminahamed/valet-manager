pub mod theme;
pub mod sidebar;
pub mod panels;

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
