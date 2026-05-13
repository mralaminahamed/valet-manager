pub mod theme;
pub mod sidebar;
pub mod panels;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedFramework {
    Laravel,
    WordPress,
    Symfony,
    Bedrock,
    Proxy,
    None,
    Other(String),
}
