use serde::Serialize;
use std::collections::{HashMap, BTreeMap};

#[derive(Serialize)]
pub struct ComposeFile {
    pub version: Option<String>,
    pub services: BTreeMap<String, Service>,
    pub networks: BTreeMap<String, Network>,
}

#[derive(Serialize)]
pub struct Service {
    pub image: String,
    pub container_name: String,
    pub restart: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<HealthCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_opt: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_add: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysctls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy: Option<Deploy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
}

#[derive(Serialize)]
pub struct HealthCheck {
    pub test: Vec<String>,
    pub interval: String,
    pub retries: u32,
    pub start_period: String,
    pub timeout: String,
}

#[derive(Serialize)]
pub struct Deploy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
}

#[derive(Serialize)]
pub struct Resources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ResourceLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<ResourceLimits>,
}

#[derive(Serialize)]
pub struct ResourceLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<String>,
}

#[derive(Serialize)]
pub struct Logging {
    pub driver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct Network {
    pub driver: String,
}
