use std::collections::HashMap;
use std::sync::Arc;
use crate::services::base::Service;
use tokio::sync::RwLock;

pub struct ServiceRegistry {
    services: RwLock<HashMap<String, Arc<dyn Service + Send + Sync>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, service: Box<dyn Service + Send + Sync>) {
        let mut services = self.services.write().await;
        let service: Arc<dyn Service + Send + Sync> = Arc::from(service);
        services.insert(service.name().to_string(), service);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Service + Send + Sync>> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    pub async fn list_services(&self) -> Vec<Arc<dyn Service + Send + Sync>> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }
}
