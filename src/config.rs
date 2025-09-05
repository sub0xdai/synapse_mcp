use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure for Synapse MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub neo4j: Neo4jConfig,
    pub server: ServerConfig,
    pub runtime: RuntimeConfig,
    pub logging: LoggingConfig,
}

/// Neo4j database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub fetch_size: usize,
    pub max_connections: usize,
}

/// Server configuration for MCP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub verbose: bool,
    pub context_file: PathBuf,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub target: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            neo4j: Neo4jConfig::default(),
            server: ServerConfig::default(),
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            uri: "bolt://localhost:7687".to_string(),
            user: "neo4j".to_string(),
            password: "password".to_string(),
            database: "neo4j".to_string(),
            fetch_size: 500,
            max_connections: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            context_file: PathBuf::from(".synapse_context"),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(), // pretty, json, compact
            target: "stdout".to_string(), // stdout, stderr
        }
    }
}

impl Config {
    /// Load configuration from multiple sources with precedence:
    /// 1. config.toml file (if exists)
    /// 2. Environment variables (SYNAPSE_*)
    /// 3. Default values
    pub fn load() -> Result<Self> {
        Self::load_from_dir(&std::env::current_dir()?)
    }

    /// Load configuration from a specific directory
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut builder = ConfigBuilder::builder();

        // Try to load from config.toml file
        let config_file = dir.join("config.toml");
        if config_file.exists() {
            builder = builder.add_source(File::from(config_file));
        }

        // Add environment variables with SYNAPSE_ prefix
        builder = builder.add_source(
            Environment::with_prefix("SYNAPSE")
                .separator("_")
                .try_parsing(true),
        );

        // Build and deserialize
        let config = builder
            .build()
            .context("Failed to build configuration")?;

        // First get defaults and then merge with loaded config
        let mut result = Config::default();
        
        // Try to deserialize the full config first
        match config.clone().try_deserialize::<Config>() {
            Ok(loaded) => {
                result.merge_with(loaded);
            }
            Err(_) => {
                // If full deserialization fails, try to load individual sections
                if let Ok(neo4j) = config.get::<Neo4jConfig>("neo4j") {
                    result.neo4j = neo4j;
                }
                if let Ok(server) = config.get::<ServerConfig>("server") {
                    result.server = server;
                }
                if let Ok(runtime) = config.get::<RuntimeConfig>("runtime") {
                    result.runtime = runtime;
                }
            }
        }
        
        // Handle direct environment variables for backward compatibility
        result.merge_env_vars()?;

        Ok(result)
    }

    /// Create a new Config for testing
    #[cfg(test)]
    pub fn for_testing() -> Self {
        Self {
            neo4j: Neo4jConfig {
                uri: "bolt://localhost:7687".to_string(),
                user: "test".to_string(),
                password: "test".to_string(),
                database: "test".to_string(),
                fetch_size: 100,
                max_connections: 5,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Use any available port
            },
            runtime: RuntimeConfig {
                verbose: true,
                context_file: PathBuf::from("/tmp/test_context"),
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "pretty".to_string(),
                target: "stdout".to_string(),
            },
        }
    }

    /// Merge this config with another, taking non-default values from other
    fn merge_with(&mut self, other: Config) {
        // For simplicity, just override with other's values
        // In a more sophisticated implementation, we could check for defaults
        self.neo4j = other.neo4j;
        self.server = other.server;
        self.runtime = other.runtime;
    }

    /// Merge environment variables for backward compatibility
    fn merge_env_vars(&mut self) -> Result<()> {
        // Neo4j environment variables
        if let Ok(uri) = std::env::var("NEO4J_URI") {
            self.neo4j.uri = uri;
        }
        if let Ok(user) = std::env::var("NEO4J_USER") {
            self.neo4j.user = user;
        }
        if let Ok(password) = std::env::var("NEO4J_PASSWORD") {
            self.neo4j.password = password;
        }
        if let Ok(database) = std::env::var("NEO4J_DATABASE") {
            self.neo4j.database = database;
        }
        if let Ok(fetch_size_str) = std::env::var("NEO4J_FETCH_SIZE") {
            self.neo4j.fetch_size = fetch_size_str.parse().unwrap_or(500);
        }
        if let Ok(max_conn_str) = std::env::var("NEO4J_MAX_CONNECTIONS") {
            self.neo4j.max_connections = max_conn_str.parse().unwrap_or(10);
        }

        // Runtime environment variables
        if let Ok(_) = std::env::var("SYNAPSE_VERBOSE") {
            self.runtime.verbose = true;
        }
        if let Ok(context_file) = std::env::var("SYNAPSE_CONTEXT_FILE") {
            self.runtime.context_file = PathBuf::from(context_file);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    use std::fs::write;
    
    // Helper to create isolated environment for testing
    fn with_isolated_env<T>(f: impl FnOnce() -> T) -> T {
        // Save original environment
        let saved_vars = [
            ("NEO4J_URI", env::var("NEO4J_URI").ok()),
            ("NEO4J_USER", env::var("NEO4J_USER").ok()),
            ("NEO4J_PASSWORD", env::var("NEO4J_PASSWORD").ok()),
            ("NEO4J_DATABASE", env::var("NEO4J_DATABASE").ok()),
            ("NEO4J_FETCH_SIZE", env::var("NEO4J_FETCH_SIZE").ok()),
            ("NEO4J_MAX_CONNECTIONS", env::var("NEO4J_MAX_CONNECTIONS").ok()),
            ("SYNAPSE_VERBOSE", env::var("SYNAPSE_VERBOSE").ok()),
            ("SYNAPSE_CONTEXT_FILE", env::var("SYNAPSE_CONTEXT_FILE").ok()),
            ("SYNAPSE_NEO4J_URI", env::var("SYNAPSE_NEO4J_URI").ok()),
            ("SYNAPSE_NEO4J_USER", env::var("SYNAPSE_NEO4J_USER").ok()),
            ("SYNAPSE_NEO4J_PASSWORD", env::var("SYNAPSE_NEO4J_PASSWORD").ok()),
            ("SYNAPSE_NEO4J_DATABASE", env::var("SYNAPSE_NEO4J_DATABASE").ok()),
            ("SYNAPSE_NEO4J_FETCH_SIZE", env::var("SYNAPSE_NEO4J_FETCH_SIZE").ok()),
            ("SYNAPSE_NEO4J_MAX_CONNECTIONS", env::var("SYNAPSE_NEO4J_MAX_CONNECTIONS").ok()),
            ("SYNAPSE_SERVER_HOST", env::var("SYNAPSE_SERVER_HOST").ok()),
            ("SYNAPSE_SERVER_PORT", env::var("SYNAPSE_SERVER_PORT").ok()),
            ("SYNAPSE_RUNTIME_VERBOSE", env::var("SYNAPSE_RUNTIME_VERBOSE").ok()),
            ("SYNAPSE_RUNTIME_CONTEXT_FILE", env::var("SYNAPSE_RUNTIME_CONTEXT_FILE").ok()),
        ];
        
        // Clear environment
        unsafe {
            for (key, _) in &saved_vars {
                env::remove_var(key);
            }
        }
        
        // Run test
        let result = f();
        
        // Restore original environment
        unsafe {
            for (key, value) in saved_vars {
                if let Some(val) = value {
                    env::set_var(key, val);
                } else {
                    env::remove_var(key);
                }
            }
        }
        
        result
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        
        assert_eq!(config.neo4j.uri, "bolt://localhost:7687");
        assert_eq!(config.neo4j.user, "neo4j");
        assert_eq!(config.neo4j.password, "password");
        assert_eq!(config.neo4j.database, "neo4j");
        assert_eq!(config.neo4j.fetch_size, 500);
        assert_eq!(config.neo4j.max_connections, 10);
        
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        
        assert_eq!(config.runtime.verbose, false);
        assert_eq!(config.runtime.context_file, PathBuf::from(".synapse_context"));
    }

    #[test]
    fn test_config_for_testing() {
        let config = Config::for_testing();
        
        assert_eq!(config.neo4j.uri, "bolt://localhost:7687");
        assert_eq!(config.neo4j.user, "test");
        assert_eq!(config.neo4j.password, "test");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 0);
        assert_eq!(config.runtime.verbose, true);
    }

    #[test]
    fn test_load_from_toml_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_file = temp_dir.path().join("config.toml");
        
        let config_content = r#"
[neo4j]
uri = "bolt://test:7687"
user = "testuser"
password = "testpass"
database = "testdb"
fetch_size = 1000
max_connections = 20

[server]
host = "0.0.0.0"
port = 9090

[runtime]
verbose = true
context_file = "/tmp/test_context"
"#;
        write(&config_file, config_content)?;
        
        let config = Config::load_from_dir(temp_dir.path())?;
        
        assert_eq!(config.neo4j.uri, "bolt://test:7687");
        assert_eq!(config.neo4j.user, "testuser");
        assert_eq!(config.neo4j.password, "testpass");
        assert_eq!(config.neo4j.database, "testdb");
        assert_eq!(config.neo4j.fetch_size, 1000);
        assert_eq!(config.neo4j.max_connections, 20);
        
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        
        assert_eq!(config.runtime.verbose, true);
        assert_eq!(config.runtime.context_file, PathBuf::from("/tmp/test_context"));
        
        Ok(())
    }

    #[test]
    fn test_load_from_environment_variables() -> Result<()> {
        with_isolated_env(|| -> Result<()> {
            let temp_dir = TempDir::new()?;
            
            // Set environment variables with both old and new format
            unsafe {
                // Old format for backward compatibility (handled by merge_env_vars)
                env::set_var("NEO4J_URI", "bolt://env:7687");
                env::set_var("NEO4J_USER", "envuser"); 
                env::set_var("NEO4J_PASSWORD", "envpass");
                env::set_var("SYNAPSE_VERBOSE", "1");
                env::set_var("SYNAPSE_CONTEXT_FILE", "/env/context");
                
                // New structured format (handled by config crate)
                env::set_var("SYNAPSE_NEO4J_URI", "bolt://env:7687");
                env::set_var("SYNAPSE_NEO4J_USER", "envuser");
                env::set_var("SYNAPSE_NEO4J_PASSWORD", "envpass");
            }
            
            let config = Config::load_from_dir(temp_dir.path())?;
            
            assert_eq!(config.neo4j.uri, "bolt://env:7687");
            assert_eq!(config.neo4j.user, "envuser");
            assert_eq!(config.neo4j.password, "envpass");
            assert_eq!(config.runtime.verbose, true);
            assert_eq!(config.runtime.context_file, PathBuf::from("/env/context"));
            
            Ok(())
        })
    }

    #[test]
    fn test_precedence_env_over_file() -> Result<()> {
        with_isolated_env(|| -> Result<()> {
            let temp_dir = TempDir::new()?;
            let config_file = temp_dir.path().join("config.toml");
            
            // Create config file
            let config_content = r#"
[neo4j]
uri = "bolt://file:7687"
user = "fileuser"
password = "filepass"
"#;
            write(&config_file, config_content)?;
            
            // Set environment variables that should override file
            unsafe {
                env::set_var("NEO4J_USER", "envuser");
                env::set_var("NEO4J_PASSWORD", "envpass");
                env::set_var("SYNAPSE_NEO4J_USER", "envuser");
                env::set_var("SYNAPSE_NEO4J_PASSWORD", "envpass");
            }
            
            let config = Config::load_from_dir(temp_dir.path())?;
            
            // Environment should override file
            assert_eq!(config.neo4j.user, "envuser");
            assert_eq!(config.neo4j.password, "envpass");
            // But file value should be preserved where no env var exists
            assert_eq!(config.neo4j.uri, "bolt://file:7687");
            
            Ok(())
        })
    }

    #[test]
    fn test_load_no_config_file() -> Result<()> {
        with_isolated_env(|| -> Result<()> {
            let temp_dir = TempDir::new()?;
            
            let config = Config::load_from_dir(temp_dir.path())?;
            
            // Should use defaults when no config file exists
            assert_eq!(config.neo4j.uri, "bolt://localhost:7687");
            assert_eq!(config.server.port, 8080);
            
            Ok(())
        })
    }
}