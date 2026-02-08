use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type NodeId = uuid::Uuid;
pub type EdgeId = uuid::Uuid;
pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Timestamp(Timestamp),
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
    Null,
}

pub type Properties = HashMap<String, PropertyValue>;

impl PropertyValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub node_type: String,
    pub properties: Properties,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Node {
    pub fn new(node_type: impl Into<String>, properties: Properties) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: NodeId::new_v4(),
            node_type: node_type.into(),
            properties,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(mut self, id: NodeId) -> Self {
        self.id = id;
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.properties.insert(key.into(), value);
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub edge_type: String,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub properties: Properties,
    pub created_at: Timestamp,
}

impl Edge {
    pub fn new(
        edge_type: impl Into<String>,
        from_node_id: NodeId,
        to_node_id: NodeId,
        properties: Properties,
    ) -> Self {
        Self {
            id: EdgeId::new_v4(),
            edge_type: edge_type.into(),
            from_node_id,
            to_node_id,
            properties,
            created_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuery {
    pub node_types: Option<Vec<String>>,
    pub edge_types: Option<Vec<String>>,
    pub property_filters: Option<HashMap<String, PropertyValue>>,
    pub limit: Option<usize>,
}

impl GraphQuery {
    pub fn new() -> Self {
        Self {
            node_types: None,
            edge_types: None,
            property_filters: None,
            limit: None,
        }
    }

    pub fn with_node_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_types = Some(vec![node_type.into()]);
        self
    }

    pub fn with_edge_type(mut self, edge_type: impl Into<String>) -> Self {
        self.edge_types = Some(vec![edge_type.into()]);
        self
    }

    pub fn with_filter(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        let mut filters = self.property_filters.unwrap_or_default();
        filters.insert(key.into(), value);
        self.property_filters = Some(filters);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for GraphQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a string ID (like "intern_0") to a deterministic UUID
/// This allows using string identifiers while maintaining UUID-based storage
pub fn string_to_node_id(s: &str) -> NodeId {
    // Use UUID v5 with a custom namespace for our application
    // The namespace is a fixed UUID: 6ba7b810-9dad-11d1-80b4-00c04fd430c8 (OID namespace)
    let namespace = uuid::Uuid::from_bytes([
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ]);
    uuid::Uuid::new_v5(&namespace, s.as_bytes())
}
