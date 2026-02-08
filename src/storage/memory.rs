use crate::domain::{Edge, EdgeId, GraphQuery, Node, NodeId};
use crate::storage::{EdgeDirection, GraphStorage, Result, StorageError, SearchQuery, SearchResults, OrderBy, OrderDirection};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct InMemoryStorage {
    nodes: Arc<RwLock<HashMap<NodeId, Node>>>,
    edges: Arc<RwLock<HashMap<EdgeId, Edge>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn matches_query(node: &Node, query: &GraphQuery) -> bool {
        // Check node type filter
        if let Some(ref types) = query.node_types {
            if !types.contains(&node.node_type) {
                return false;
            }
        }

        // Check property filters
        if let Some(ref filters) = query.property_filters {
            for (key, expected_value) in filters {
                match node.properties.get(key) {
                    Some(actual_value) if actual_value == expected_value => continue,
                    _ => return false,
                }
            }
        }

        true
    }
    
    fn matches_search_query(node: &Node, query: &SearchQuery) -> bool {
        // Check node types
        if !query.node_types.is_empty() && !query.node_types.contains(&node.node_type) {
            return false;
        }
        
        // Check text search
        if let Some(ref search_text) = query.search_text {
            let search_lower = search_text.to_lowercase();
            let node_text = serde_json::to_string(&node.properties).unwrap_or_default().to_lowercase();
            if !node_text.contains(&search_lower) {
                return false;
            }
        }
        
        // Check created time range
        if let Some(after) = query.created_after {
            if node.created_at < after {
                return false;
            }
        }
        if let Some(before) = query.created_before {
            if node.created_at > before {
                return false;
            }
        }
        
        // Check updated time range
        if let Some(after) = query.updated_after {
            if node.updated_at < after {
                return false;
            }
        }
        
        // Check property filters
        for (key, value) in &query.property_filters {
            match node.properties.get(key) {
                Some(prop_val) => {
                    let prop_str = serde_json::to_string(prop_val).unwrap_or_default();
                    let value_str = format!("\"{}\"", value);
                    if prop_str != value_str && prop_str != *value {
                        return false;
                    }
                }
                None => return false,
            }
        }
        
        true
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphStorage for InMemoryStorage {
    async fn create_node(&self, node: &Node) -> Result<Node> {
        let mut nodes = self.nodes.write().await;
        if nodes.contains_key(&node.id) {
            return Err(StorageError::ConstraintViolation(
                format!("Node with ID {} already exists", node.id)
            ));
        }
        nodes.insert(node.id, node.clone());
        Ok(node.clone())
    }

    async fn get_node(&self, id: NodeId) -> Result<Node> {
        let nodes = self.nodes.read().await;
        nodes.get(&id)
            .cloned()
            .ok_or(StorageError::NodeNotFound(id))
    }

    async fn update_node(&self, node: &Node) -> Result<Node> {
        let mut nodes = self.nodes.write().await;
        if !nodes.contains_key(&node.id) {
            return Err(StorageError::NodeNotFound(node.id));
        }
        nodes.insert(node.id, node.clone());
        Ok(node.clone())
    }

    async fn delete_node(&self, id: NodeId) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        let mut edges = self.edges.write().await;
        
        if !nodes.contains_key(&id) {
            return Err(StorageError::NodeNotFound(id));
        }
        
        // Remove all edges connected to this node
        edges.retain(|_, edge| {
            edge.from_node_id != id && edge.to_node_id != id
        });
        
        nodes.remove(&id);
        Ok(())
    }

    async fn query_nodes(&self, query: &GraphQuery) -> Result<Vec<Node>> {
        let nodes = self.nodes.read().await;
        let mut results: Vec<Node> = nodes
            .values()
            .filter(|node| Self::matches_query(node, query))
            .cloned()
            .collect();
        
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }

    async fn create_edge(&self, edge: &Edge) -> Result<Edge> {
        let nodes = self.nodes.read().await;
        
        // Verify both nodes exist
        if !nodes.contains_key(&edge.from_node_id) {
            return Err(StorageError::NodeNotFound(edge.from_node_id));
        }
        if !nodes.contains_key(&edge.to_node_id) {
            return Err(StorageError::NodeNotFound(edge.to_node_id));
        }
        
        drop(nodes);
        
        let mut edges = self.edges.write().await;
        edges.insert(edge.id, edge.clone());
        Ok(edge.clone())
    }

    async fn get_edge(&self, id: EdgeId) -> Result<Edge> {
        let edges = self.edges.read().await;
        edges.get(&id)
            .cloned()
            .ok_or(StorageError::EdgeNotFound(id))
    }

    async fn delete_edge(&self, id: EdgeId) -> Result<()> {
        let mut edges = self.edges.write().await;
        if !edges.contains_key(&id) {
            return Err(StorageError::EdgeNotFound(id));
        }
        edges.remove(&id);
        Ok(())
    }

    async fn get_edges_from(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>> {
        let edges = self.edges.read().await;
        let results: Vec<Edge> = edges
            .values()
            .filter(|edge| {
                edge.from_node_id == node_id &&
                edge_type.map_or(true, |et| edge.edge_type == et)
            })
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_edges_to(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>> {
        let edges = self.edges.read().await;
        let results: Vec<Edge> = edges
            .values()
            .filter(|edge| {
                edge.to_node_id == node_id &&
                edge_type.map_or(true, |et| edge.edge_type == et)
            })
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_neighbors(
        &self,
        node_id: NodeId,
        edge_type: Option<&str>,
        direction: EdgeDirection,
    ) -> Result<Vec<Node>> {
        let edges = self.edges.read().await;
        let nodes = self.nodes.read().await;
        
        let mut neighbor_ids: Vec<NodeId> = Vec::new();
        
        for edge in edges.values() {
            let matches_type = edge_type.map_or(true, |et| edge.edge_type == et);
            
            match direction {
                EdgeDirection::Outgoing if edge.from_node_id == node_id && matches_type => {
                    neighbor_ids.push(edge.to_node_id);
                }
                EdgeDirection::Incoming if edge.to_node_id == node_id && matches_type => {
                    neighbor_ids.push(edge.from_node_id);
                }
                EdgeDirection::Both if matches_type && 
                    (edge.from_node_id == node_id || edge.to_node_id == node_id) => {
                    let neighbor_id = if edge.from_node_id == node_id {
                        edge.to_node_id
                    } else {
                        edge.from_node_id
                    };
                    neighbor_ids.push(neighbor_id);
                }
                _ => {}
            }
        }
        
        let neighbors: Vec<Node> = neighbor_ids
            .into_iter()
            .filter_map(|id| nodes.get(&id).cloned())
            .collect();
        
        Ok(neighbors)
    }
    
    async fn search_nodes(&self, query: &SearchQuery) -> Result<SearchResults<Node>> {
        let nodes = self.nodes.read().await;
        
        // Filter nodes based on query criteria
        let mut results: Vec<Node> = nodes.values()
            .filter(|node| Self::matches_search_query(node, query))
            .cloned()
            .collect();
        
        // Sort results
        results.sort_by(|a, b| {
            let cmp = match query.order_by {
                OrderBy::CreatedAt => a.created_at.cmp(&b.created_at),
                OrderBy::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                OrderBy::Relevance => a.updated_at.cmp(&b.updated_at), // Fallback
            };
            
            match query.order_direction {
                OrderDirection::Asc => cmp,
                OrderDirection::Desc => cmp.reverse(),
            }
        });
        
        let total_count = results.len();
        let offset = query.offset;
        let limit = query.limit;
        
        // Check if there are more results
        let has_more = results.len() > offset + limit;
        
        // Apply pagination
        let paginated: Vec<Node> = results.into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        let returned_count = paginated.len();
        
        Ok(SearchResults {
            items: paginated,
            total_count,
            returned_count,
            has_more,
            limit,
            offset,
        })
    }
    
    async fn count_nodes(&self, query: &SearchQuery) -> Result<usize> {
        let nodes = self.nodes.read().await;
        
        let count = nodes.values()
            .filter(|node| Self::matches_search_query(node, query))
            .count();
        
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Properties;

    #[tokio::test]
    async fn test_create_and_get_node() {
        let storage = InMemoryStorage::new();
        let node = Node::new("test", Properties::new());
        
        let created = storage.create_node(&node).await.unwrap();
        assert_eq!(created.id, node.id);
        
        let retrieved = storage.get_node(node.id).await.unwrap();
        assert_eq!(retrieved.id, node.id);
    }

    #[tokio::test]
    async fn test_create_edge_between_nodes() {
        let storage = InMemoryStorage::new();
        
        let node1 = Node::new("agent", Properties::new());
        let node2 = Node::new("mailbox", Properties::new());
        
        storage.create_node(&node1).await.unwrap();
        storage.create_node(&node2).await.unwrap();
        
        let edge = Edge::new("owns", node1.id, node2.id, Properties::new());
        let created = storage.create_edge(&edge).await.unwrap();
        
        assert_eq!(created.from_node_id, node1.id);
        assert_eq!(created.to_node_id, node2.id);
    }

    #[tokio::test]
    async fn test_query_nodes_with_type_filter() {
        let storage = InMemoryStorage::new();
        
        let agent = Node::new("agent", Properties::new());
        let mailbox = Node::new("mailbox", Properties::new());
        
        storage.create_node(&agent).await.unwrap();
        storage.create_node(&mailbox).await.unwrap();
        
        let query = GraphQuery::new().with_node_type("agent");
        let results = storage.query_nodes(&query).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_type, "agent");
    }

    #[tokio::test]
    async fn test_get_neighbors() {
        let storage = InMemoryStorage::new();
        
        let agent = Node::new("agent", Properties::new());
        let mailbox1 = Node::new("mailbox", Properties::new());
        let mailbox2 = Node::new("mailbox", Properties::new());
        
        storage.create_node(&agent).await.unwrap();
        storage.create_node(&mailbox1).await.unwrap();
        storage.create_node(&mailbox2).await.unwrap();
        
        let edge1 = Edge::new("owns", agent.id, mailbox1.id, Properties::new());
        let edge2 = Edge::new("owns", agent.id, mailbox2.id, Properties::new());
        
        storage.create_edge(&edge1).await.unwrap();
        storage.create_edge(&edge2).await.unwrap();
        
        let neighbors = storage.get_neighbors(agent.id, Some("owns"), EdgeDirection::Outgoing).await.unwrap();
        assert_eq!(neighbors.len(), 2);
    }
}
