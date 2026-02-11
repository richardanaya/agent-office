use crate::domain::{Edge, GraphQuery, Node, NodeId};
use async_trait::async_trait;
use thiserror::Error;
use chrono::{DateTime, Utc};

/// Advanced search query with full-text, time range, and pagination support
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Node types to search (e.g., "mail", "note", "agent")
    pub node_types: Vec<String>,
    /// Full-text search pattern (PostgreSQL tsquery compatible)
    pub search_text: Option<String>,
    /// Created/modified after this time
    pub created_after: Option<DateTime<Utc>>,
    /// Created/modified before this time
    pub created_before: Option<DateTime<Utc>>,
    /// Updated after this time
    pub updated_after: Option<DateTime<Utc>>,
    /// Property filters
    pub property_filters: Vec<(String, String)>,
    /// Maximum results to return
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
}



impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            node_types: vec![],
            search_text: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            property_filters: vec![],
            limit: 50,
            offset: 0,
        }
    }
}

/// Search results with pagination info
#[derive(Debug, Clone)]
pub struct SearchResults<T> {
    pub items: Vec<T>,
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[async_trait]
pub trait GraphStorage: Send + Sync {
    // Node operations
    async fn create_node(&self, node: &Node) -> Result<Node>;
    async fn get_node(&self, id: NodeId) -> Result<Node>;
    async fn update_node(&self, node: &Node) -> Result<Node>;
    async fn delete_node(&self, id: NodeId) -> Result<()>;
    async fn query_nodes(&self, query: &GraphQuery) -> Result<Vec<Node>>;
    
    // Edge operations
    async fn create_edge(&self, edge: &Edge) -> Result<Edge>;
    async fn get_edges_from(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>>;
    async fn get_edges_to(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>>;
    
    // Graph traversal
    async fn get_neighbors(
        &self,
        node_id: NodeId,
        edge_type: Option<&str>,
        direction: EdgeDirection,
    ) -> Result<Vec<Node>>;
    
    // Advanced search with full-text, time range, and pagination
    async fn search_nodes(&self, query: &SearchQuery) -> Result<SearchResults<Node>>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeDirection {
    Outgoing,
    Incoming,
}

pub mod memory;
pub mod postgres;
