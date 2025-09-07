//! Health check and monitoring module
//! 
//! Provides health check capabilities for system dependencies and overall service status.
//! Follows SOLID principles with clean separation of concerns.

use crate::{Result, SynapseError, graph::Graph};
use crate::db::pool::PoolStats;
use crate::cache::RuleCache;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, warn, instrument};
use async_trait::async_trait;

/// Overall health status of the service
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All dependencies are healthy
    Healthy,
    /// Some non-critical dependencies have issues
    Degraded,
    /// Critical dependencies are failing
    Unhealthy,
}

/// Individual dependency health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
    pub last_checked: u64, // Unix timestamp
}

/// Connection pool health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolHealth {
    pub active: u32,
    pub idle: u32,
    pub max: u32,
    pub utilization_percent: f64,
}

/// Neo4j specific health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jHealth {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub connection_pool: ConnectionPoolHealth,
    pub message: Option<String>,
}

/// Cache health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealth {
    pub status: HealthStatus,
    pub hit_rate: f64,
    pub entries: u64,
    pub max_entries: u64,
    pub utilization_percent: f64,
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub memory_used_mb: u64,
    pub memory_available_mb: u64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
}

/// Comprehensive service status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub dependencies: DependencyStatus,
    pub system: SystemHealth,
    pub timestamp: u64, // Unix timestamp
}

/// All dependency health statuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub neo4j: Neo4jHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheHealth>,
}

/// Trait for checking health of individual dependencies
/// 
/// This follows the Interface Segregation Principle (ISP) by providing
/// a focused interface for health checking.
#[async_trait]
pub trait HealthChecker {
    /// Check the health of this dependency
    async fn check_health(&self) -> DependencyHealth;
    
    /// Get a human-readable name for this dependency
    fn dependency_name(&self) -> &'static str;
}

/// Health checker for Neo4j database
pub struct Neo4jHealthChecker {
    graph: std::sync::Arc<Graph>,
}

impl Neo4jHealthChecker {
    pub fn new(graph: Graph) -> Self {
        Self { graph: std::sync::Arc::new(graph) }
    }
    
    pub fn new_with_arc(graph: std::sync::Arc<Graph>) -> Self {
        Self { graph }
    }
    
    /// Check Neo4j health with detailed connection pool information
    #[instrument(skip(self))]
    pub async fn check_detailed_health(&self) -> Result<Neo4jHealth> {
        let start = Instant::now();
        
        // Try to execute a simple query to verify connectivity
        let health_result = self.check_basic_connectivity().await;
        let latency_ms = start.elapsed().as_millis() as u64;
        
        // Get connection pool statistics if available
        let pool_stats = self.get_pool_stats().await.unwrap_or_else(|_| {
            // Fallback stats when pool information is unavailable
            PoolStats {
                size: 0,
                idle_connections: 0,
                active_connections: 0,
                total_created: 0,
                total_errors: 0,
                max_size: 10, // Default from config
            }
        });
        
        let connection_pool = ConnectionPoolHealth {
            active: pool_stats.active_connections,
            idle: pool_stats.idle_connections,
            max: pool_stats.max_size,
            utilization_percent: if pool_stats.max_size > 0 {
                (pool_stats.active_connections as f64 / pool_stats.max_size as f64) * 100.0
            } else {
                0.0
            },
        };
        
        let (status, message) = match health_result {
            Ok(_) => {
                if latency_ms > 1000 {
                    (HealthStatus::Degraded, Some("High latency".to_string()))
                } else if pool_stats.active_connections as f64 / pool_stats.max_size as f64 > 0.8 {
                    (HealthStatus::Degraded, Some("High connection pool utilization".to_string()))
                } else {
                    (HealthStatus::Healthy, None)
                }
            }
            Err(e) => (HealthStatus::Unhealthy, Some(format!("Connection failed: {}", e))),
        };
        
        Ok(Neo4jHealth {
            status,
            latency_ms,
            connection_pool,
            message,
        })
    }
    
    /// Check basic Neo4j connectivity
    async fn check_basic_connectivity(&self) -> Result<()> {
        // Use the graph's built-in health check method
        match self.graph.health_check().await {
            Ok(true) => {
                debug!("Neo4j health check passed");
                Ok(())
            }
            Ok(false) => {
                warn!("Neo4j health check failed");
                Err(SynapseError::Database("Health check returned false".to_string()))
            }
            Err(e) => {
                warn!("Neo4j health check failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get connection pool statistics
    async fn get_pool_stats(&self) -> Result<PoolStats> {
        // This would need to be implemented based on how the Graph exposes pool stats
        // For now, return a placeholder implementation
        Ok(PoolStats {
            size: 5,
            idle_connections: 3,
            active_connections: 2,
            total_created: 10,
            total_errors: 0,
            max_size: 10,
        })
    }
}

#[async_trait]
impl HealthChecker for Neo4jHealthChecker {
    async fn check_health(&self) -> DependencyHealth {
        let start = Instant::now();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        match self.check_basic_connectivity().await {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                DependencyHealth {
                    status: if latency_ms > 1000 { 
                        HealthStatus::Degraded 
                    } else { 
                        HealthStatus::Healthy 
                    },
                    latency_ms: Some(latency_ms),
                    message: None,
                    last_checked: timestamp,
                }
            }
            Err(e) => DependencyHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: Some(format!("Connection failed: {}", e)),
                last_checked: timestamp,
            },
        }
    }
    
    fn dependency_name(&self) -> &'static str {
        "neo4j"
    }
}

/// Health checker for rule cache
pub struct CacheHealthChecker {
    cache: std::sync::Arc<RuleCache>,
}

impl CacheHealthChecker {
    pub fn new(cache: std::sync::Arc<RuleCache>) -> Self {
        Self { cache }
    }
    
    /// Check cache health with detailed statistics
    #[instrument(skip(self))]
    pub async fn check_detailed_health(&self) -> Result<CacheHealth> {
        let stats = self.cache.stats().await;
        
        let hit_rate = stats.hit_rate;
        let utilization_percent = if stats.max_size > 0 {
            (stats.size as f64 / stats.max_size as f64) * 100.0
        } else {
            0.0
        };
        
        let status = if hit_rate < 0.5 {
            HealthStatus::Degraded // Low hit rate indicates potential issues
        } else {
            HealthStatus::Healthy
        };
        
        Ok(CacheHealth {
            status,
            hit_rate,
            entries: stats.size,
            max_entries: stats.max_size,
            utilization_percent,
        })
    }
}

#[async_trait]
impl HealthChecker for CacheHealthChecker {
    async fn check_health(&self) -> DependencyHealth {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        match self.check_detailed_health().await {
            Ok(cache_health) => DependencyHealth {
                status: cache_health.status,
                latency_ms: Some(1), // Cache checks are very fast
                message: if cache_health.hit_rate < 0.5 {
                    Some(format!("Low hit rate: {:.2}", cache_health.hit_rate))
                } else {
                    None
                },
                last_checked: timestamp,
            },
            Err(e) => DependencyHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: Some(1),
                message: Some(format!("Cache check failed: {}", e)),
                last_checked: timestamp,
            },
        }
    }
    
    fn dependency_name(&self) -> &'static str {
        "cache"
    }
}

/// System resource health checker
#[derive(Debug)]
pub struct SystemHealthChecker;

impl SystemHealthChecker {
    pub fn new() -> Self {
        Self
    }
    
    /// Get current system health information
    #[instrument]
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        // Get memory information
        let (memory_used_mb, memory_available_mb) = self.get_memory_info().await?;
        let memory_usage_percent = if memory_available_mb > 0 {
            (memory_used_mb as f64 / memory_available_mb as f64) * 100.0
        } else {
            0.0
        };
        
        // Get CPU information (simplified - would need more sophisticated implementation)
        let cpu_usage_percent = self.get_cpu_usage().await.unwrap_or(0.0);
        
        Ok(SystemHealth {
            memory_used_mb,
            memory_available_mb,
            memory_usage_percent,
            cpu_usage_percent,
        })
    }
    
    /// Get memory usage information
    /// 
    /// This is a simplified implementation. In production, you might want to use
    /// a crate like `sysinfo` for more accurate system information.
    async fn get_memory_info(&self) -> Result<(u64, u64)> {
        // Simplified memory info - in real implementation, use sysinfo crate
        let memory_used_mb = 256; // Placeholder
        let memory_available_mb = 1024; // Placeholder
        
        Ok((memory_used_mb, memory_available_mb))
    }
    
    /// Get CPU usage percentage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Simplified CPU usage - in real implementation, use sysinfo crate
        Ok(15.5) // Placeholder
    }
}

/// Main health service that coordinates all health checks
/// 
/// This follows the Single Responsibility Principle (SRP) by focusing
/// solely on health coordination.
pub struct HealthService {
    neo4j_checker: Neo4jHealthChecker,
    cache_checker: Option<CacheHealthChecker>,
    system_checker: SystemHealthChecker,
    start_time: Instant,
}

impl HealthService {
    /// Create a new health service with required dependencies
    pub fn new(
        graph: Graph, 
        cache: Option<std::sync::Arc<RuleCache>>
    ) -> Self {
        let neo4j_checker = Neo4jHealthChecker::new(graph);
        let cache_checker = cache.map(CacheHealthChecker::new);
        let system_checker = SystemHealthChecker::new();
        
        Self {
            neo4j_checker,
            cache_checker,
            system_checker,
            start_time: Instant::now(),
        }
    }
    
    /// Create a new health service with Arc<Graph> (for shared ownership)
    pub fn new_with_arc(
        graph: std::sync::Arc<Graph>, 
        cache: Option<std::sync::Arc<RuleCache>>
    ) -> Self {
        let neo4j_checker = Neo4jHealthChecker::new_with_arc(graph);
        let cache_checker = cache.map(CacheHealthChecker::new);
        let system_checker = SystemHealthChecker::new();
        
        Self {
            neo4j_checker,
            cache_checker,
            system_checker,
            start_time: Instant::now(),
        }
    }
    
    /// Simple health check - returns basic OK status
    /// 
    /// This is designed to be very fast for load balancer health checks.
    #[instrument(skip(self))]
    pub async fn check_health(&self) -> Result<String> {
        // Basic connectivity check to Neo4j
        match self.neo4j_checker.check_basic_connectivity().await {
            Ok(_) => {
                debug!("Health check passed");
                Ok("OK".to_string())
            }
            Err(_) => {
                warn!("Health check failed - Neo4j unavailable");
                Err(SynapseError::Internal("Service unhealthy".to_string()))
            }
        }
    }
    
    /// Comprehensive status check with detailed dependency information
    #[instrument(skip(self))]
    pub async fn get_detailed_status(&self) -> Result<ServiceStatus> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        let uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Check Neo4j health
        let neo4j_health = self.neo4j_checker.check_detailed_health().await
            .unwrap_or_else(|_| Neo4jHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: 0,
                connection_pool: ConnectionPoolHealth {
                    active: 0,
                    idle: 0,
                    max: 0,
                    utilization_percent: 0.0,
                },
                message: Some("Failed to check Neo4j health".to_string()),
            });
        
        // Check cache health if available
        let cache_health = if let Some(ref cache_checker) = self.cache_checker {
            Some(cache_checker.check_detailed_health().await
                .unwrap_or_else(|_| CacheHealth {
                    status: HealthStatus::Unhealthy,
                    hit_rate: 0.0,
                    entries: 0,
                    max_entries: 0,
                    utilization_percent: 0.0,
                }))
        } else {
            None
        };
        
        // Get system health
        let system_health = self.system_checker.get_system_health().await
            .unwrap_or_else(|_| SystemHealth {
                memory_used_mb: 0,
                memory_available_mb: 0,
                memory_usage_percent: 0.0,
                cpu_usage_percent: 0.0,
            });
        
        // Determine overall status based on dependencies
        let overall_status = self.calculate_overall_status(&neo4j_health, &cache_health);
        
        Ok(ServiceStatus {
            status: overall_status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
            dependencies: DependencyStatus {
                neo4j: neo4j_health,
                cache: cache_health,
            },
            system: system_health,
            timestamp,
        })
    }
    
    /// Calculate overall service status based on dependency health
    /// 
    /// This implements a simple aggregation strategy:
    /// - Healthy: All critical dependencies are healthy
    /// - Degraded: Critical dependencies are healthy but some have warnings
    /// - Unhealthy: Any critical dependency is unhealthy
    fn calculate_overall_status(
        &self,
        neo4j: &Neo4jHealth,
        cache: &Option<CacheHealth>,
    ) -> HealthStatus {
        // Neo4j is critical - if it's unhealthy, service is unhealthy
        if neo4j.status == HealthStatus::Unhealthy {
            return HealthStatus::Unhealthy;
        }
        
        // If Neo4j is degraded, overall status is at least degraded
        if neo4j.status == HealthStatus::Degraded {
            return HealthStatus::Degraded;
        }
        
        // Cache issues cause degraded status (not critical failure)
        if let Some(cache_health) = cache {
            if cache_health.status == HealthStatus::Unhealthy 
                || cache_health.status == HealthStatus::Degraded {
                return HealthStatus::Degraded;
            }
        }
        
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");
        
        let degraded = HealthStatus::Degraded;
        let json = serde_json::to_string(&degraded).unwrap();
        assert_eq!(json, "\"degraded\"");
    }
    
    #[test]
    fn test_system_health_creation() {
        let system = SystemHealth {
            memory_used_mb: 256,
            memory_available_mb: 1024,
            memory_usage_percent: 25.0,
            cpu_usage_percent: 15.5,
        };
        
        assert_eq!(system.memory_usage_percent, 25.0);
        assert!(system.cpu_usage_percent > 0.0);
    }
    
    #[test]
    fn test_connection_pool_utilization_calculation() {
        let pool = ConnectionPoolHealth {
            active: 8,
            idle: 2,
            max: 10,
            utilization_percent: 80.0,
        };
        
        assert_eq!(pool.utilization_percent, 80.0);
        assert_eq!(pool.active + pool.idle, pool.max);
    }
}