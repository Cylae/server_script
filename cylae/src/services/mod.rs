use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use crate::core::hardware::HardwareProfile;
use std::sync::Arc;

pub mod plex;

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    fn pretty_name(&self) -> &'static str;

    fn get_url(&self) -> Option<String> { None }
    fn firewall_ports(&self) -> Vec<String> { vec![] }

    async fn install(&self, profile: &HardwareProfile) -> Result<()>;
}

pub struct ServiceRegistry {
    services: HashMap<&'static str, Arc<dyn Service>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, service: Arc<dyn Service>) {
        self.services.insert(service.name(), service);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        self.services.get(name).cloned()
    }

    pub fn get_all(&self) -> Vec<Arc<dyn Service>> {
        self.services.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockService;

    #[async_trait]
    impl Service for MockService {
        fn name(&self) -> &'static str { "mock" }
        fn pretty_name(&self) -> &'static str { "Mock Service" }
        async fn install(&self, _profile: &HardwareProfile) -> Result<()> { Ok(()) }
    }

    #[test]
    fn test_registry() {
        let mut registry = ServiceRegistry::new();
        registry.register(Arc::new(MockService));

        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());
        assert_eq!(registry.get_all().len(), 1);
    }
}
