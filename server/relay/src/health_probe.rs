use crate::repo::PgRepositoryCore;
use crate::task_event_consumer::{RabbitMQTaskEventCore, TaskEventCore};
use std::sync::Arc;

/// Represents the health status of an individual service component
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub component: String,
    pub message: String,
}

/// Centralized service for performing health checks on system components
#[derive(Clone)]
pub struct ServiceHealthProbe {
    repository_core: PgRepositoryCore,
    broker_core: Option<Arc<RabbitMQTaskEventCore>>,
}

impl ServiceHealthProbe {
    pub fn new(
        repository_core: PgRepositoryCore,
        broker_core: Option<Arc<RabbitMQTaskEventCore>>,
    ) -> Self {
        Self {
            repository_core,
            broker_core,
        }
    }

    /// Checks health of all configured system components
    ///
    /// Returns a tuple containing overall health status and detailed reports for each component
    pub async fn check_health(&self) -> (bool, Vec<ServiceHealth>) {
        let mut reports = Vec::new();
        let mut is_system_healthy = true;

        // Check database health
        let db_health = match self.repository_core.health_check().await {
            Ok(_) => ServiceHealth {
                is_healthy: true,
                component: "database".to_string(),
                message: "Database connection is healthy".to_string(),
            },
            Err(e) => {
                is_system_healthy = false;
                ServiceHealth {
                    is_healthy: false,
                    component: "database".to_string(),
                    message: format!("Database is unavailable: {}", e),
                }
            }
        };
        reports.push(db_health);

        // Check broker health if configured
        if let Some(broker) = &self.broker_core {
            let broker_health = match broker.health_check().await {
                Ok(_) => ServiceHealth {
                    is_healthy: true,
                    component: "broker".to_string(),
                    message: "Message broker connection is healthy".to_string(),
                },
                Err(e) => {
                    is_system_healthy = false;
                    ServiceHealth {
                        is_healthy: false,
                        component: "broker".to_string(),
                        message: format!("Message broker is unavailable: {}", e),
                    }
                }
            };
            reports.push(broker_health);
        }

        (is_system_healthy, reports)
    }
}
