//! Database connection pooling module
//! 
//! This module provides connection pooling functionality for Neo4j
//! using the bb8 connection pool library.

pub mod connection_manager;
pub mod pool;

pub use connection_manager::Neo4jConnectionManager;
pub use pool::{ConnectionPool, PoolStats, PoolError};

// Re-export common types for convenience
pub use bb8::{Pool, PooledConnection};