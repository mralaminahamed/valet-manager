#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ManagedService {
    pub name: String,
    pub display_name: String,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub unread_count: Option<u32>,
}
