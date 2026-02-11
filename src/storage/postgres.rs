use crate::domain::{Edge, GraphQuery, Node, NodeId, Properties};
use crate::storage::{EdgeDirection, GraphStorage, Result, StorageError, SearchQuery, SearchResults};
use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};

pub struct PostgresStorage {
    pool: Pool<Postgres>,
}

impl PostgresStorage {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn setup_tables(&self) -> Result<()> {
        // Execute each statement separately since SQLx doesn't support multiple statements in one query
        
        // Drop existing tables
        sqlx::query("DROP TABLE IF EXISTS edges CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("DROP TABLE IF EXISTS nodes CASCADE")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Create nodes table
        sqlx::query(
            r#"
            CREATE TABLE nodes (
                id UUID PRIMARY KEY,
                node_type VARCHAR(255) NOT NULL,
                properties JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Create edges table
        sqlx::query(
            r#"
            CREATE TABLE edges (
                id UUID PRIMARY KEY,
                edge_type VARCHAR(255) NOT NULL,
                from_node_id UUID NOT NULL,
                to_node_id UUID NOT NULL,
                properties JSONB NOT NULL DEFAULT '{}',
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                FOREIGN KEY (from_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
                FOREIGN KEY (to_node_id) REFERENCES nodes(id) ON DELETE CASCADE
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Create indexes for basic lookups
        sqlx::query("CREATE INDEX idx_nodes_type ON nodes(node_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_nodes_properties ON nodes USING GIN(properties)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Create time-based indexes for recency searches
        sqlx::query("CREATE INDEX idx_nodes_created_at ON nodes(created_at DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_nodes_updated_at ON nodes(updated_at DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Create composite indexes for time range + type queries
        sqlx::query("CREATE INDEX idx_nodes_type_created ON nodes(node_type, created_at DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_nodes_type_updated ON nodes(node_type, updated_at DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_edges_type ON edges(edge_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_edges_from ON edges(from_node_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_edges_to ON edges(to_node_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_edges_from_type ON edges(from_node_id, edge_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        sqlx::query("CREATE INDEX idx_edges_to_type ON edges(to_node_id, edge_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Create indexes on edge timestamps
        sqlx::query("CREATE INDEX idx_edges_created_at ON edges(created_at DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }
    
    /// Helper function to convert properties JSONB to searchable text
    #[allow(dead_code)]
    fn properties_to_search_text(properties: &Properties) -> String {
        let mut texts = Vec::new();
        for (_, value) in properties {
            if let serde_json::Value::String(s) = serde_json::to_value(value).unwrap_or_default() {
                texts.push(s);
            }
        }
        texts.join(" ")
    }
}

#[async_trait]
impl GraphStorage for PostgresStorage {
    async fn create_node(&self, node: &Node) -> Result<Node> {
        let properties_json = serde_json::to_value(&node.properties)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO nodes (id, node_type, properties, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(node.id)
        .bind(&node.node_type)
        .bind(properties_json)
        .bind(node.created_at)
        .bind(node.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(node.clone())
    }

    async fn get_node(&self, id: NodeId) -> Result<Node> {
        let row = sqlx::query(
            r#"
            SELECT id, node_type, properties, created_at, updated_at
            FROM nodes
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let properties_json: serde_json::Value = row.try_get("properties")
                    .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
                let properties = serde_json::from_value(properties_json)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                Ok(Node {
                    id: row.try_get("id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                    node_type: row.try_get("node_type").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                    properties,
                    created_at: row.try_get("created_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                    updated_at: row.try_get("updated_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                })
            }
            None => Err(StorageError::NodeNotFound(id)),
        }
    }

    async fn update_node(&self, node: &Node) -> Result<Node> {
        let properties_json = serde_json::to_value(&node.properties)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            r#"
            UPDATE nodes
            SET node_type = $2, properties = $3, updated_at = $4
            WHERE id = $1
            "#
        )
        .bind(node.id)
        .bind(&node.node_type)
        .bind(properties_json)
        .bind(node.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NodeNotFound(node.id));
        }

        Ok(node.clone())
    }

    async fn delete_node(&self, id: NodeId) -> Result<()> {
        let result = sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NodeNotFound(id));
        }

        Ok(())
    }

    async fn query_nodes(&self, query: &GraphQuery) -> Result<Vec<Node>> {
        let mut sql = String::from("SELECT id, node_type, properties, created_at, updated_at FROM nodes WHERE 1=1");
        
        // Handle node_types with IN clause instead of ANY for better compatibility
        if let Some(ref types) = query.node_types {
            if types.len() == 1 {
                // Single type - use direct equality
                sql.push_str(&format!(" AND node_type = '{}'", types[0]));
            } else if !types.is_empty() {
                // Multiple types - use IN clause
                let type_list: Vec<String> = types.iter()
                    .map(|t| format!("'{}'", t.replace("'", "''")))
                    .collect();
                sql.push_str(&format!(" AND node_type IN ({})", type_list.join(", ")));
            }
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            let properties_json: serde_json::Value = row.try_get("properties")
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let properties = serde_json::from_value(properties_json)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            nodes.push(Node {
                id: row.try_get("id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                node_type: row.try_get("node_type").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                properties,
                created_at: row.try_get("created_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
            });
        }

        Ok(nodes)
    }

    async fn create_edge(&self, edge: &Edge) -> Result<Edge> {
        let properties_json = serde_json::to_value(&edge.properties)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO edges (id, edge_type, from_node_id, to_node_id, properties, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(edge.id)
        .bind(&edge.edge_type)
        .bind(edge.from_node_id)
        .bind(edge.to_node_id)
        .bind(properties_json)
        .bind(edge.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(edge.clone())
    }

    async fn get_edges_from(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>> {
        let rows = if let Some(et) = edge_type {
            sqlx::query(
                r#"
                SELECT id, edge_type, from_node_id, to_node_id, properties, created_at
                FROM edges
                WHERE from_node_id = $1 AND edge_type = $2
                ORDER BY created_at DESC
                "#
            )
            .bind(node_id)
            .bind(et)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, edge_type, from_node_id, to_node_id, properties, created_at
                FROM edges
                WHERE from_node_id = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(node_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut edges = Vec::new();
        for row in rows {
            let properties_json: serde_json::Value = row.try_get("properties")
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let properties = serde_json::from_value(properties_json)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            edges.push(Edge {
                id: row.try_get("id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                edge_type: row.try_get("edge_type").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                from_node_id: row.try_get("from_node_id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                to_node_id: row.try_get("to_node_id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                properties,
                created_at: row.try_get("created_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
            });
        }

        Ok(edges)
    }

    async fn get_edges_to(&self, node_id: NodeId, edge_type: Option<&str>) -> Result<Vec<Edge>> {
        let rows = if let Some(et) = edge_type {
            sqlx::query(
                r#"
                SELECT id, edge_type, from_node_id, to_node_id, properties, created_at
                FROM edges
                WHERE to_node_id = $1 AND edge_type = $2
                ORDER BY created_at DESC
                "#
            )
            .bind(node_id)
            .bind(et)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, edge_type, from_node_id, to_node_id, properties, created_at
                FROM edges
                WHERE to_node_id = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(node_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut edges = Vec::new();
        for row in rows {
            let properties_json: serde_json::Value = row.try_get("properties")
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let properties = serde_json::from_value(properties_json)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            edges.push(Edge {
                id: row.try_get("id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                edge_type: row.try_get("edge_type").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                from_node_id: row.try_get("from_node_id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                to_node_id: row.try_get("to_node_id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                properties,
                created_at: row.try_get("created_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
            });
        }

        Ok(edges)
    }

    async fn get_neighbors(
        &self,
        node_id: NodeId,
        edge_type: Option<&str>,
        direction: EdgeDirection,
    ) -> Result<Vec<Node>> {
        let mut neighbors = Vec::new();

        match direction {
            EdgeDirection::Outgoing => {
                let edges = self.get_edges_from(node_id, edge_type).await?;
                for edge in edges {
                    if let Ok(node) = self.get_node(edge.to_node_id).await {
                        neighbors.push(node);
                    }
                }
            }
            _ => {}
        }

        match direction {
            EdgeDirection::Incoming => {
                let edges = self.get_edges_to(node_id, edge_type).await?;
                for edge in edges {
                    if let Ok(node) = self.get_node(edge.from_node_id).await {
                        neighbors.push(node);
                    }
                }
            }
            _ => {}
        }

        Ok(neighbors)
    }

    async fn search_nodes(&self, query: &SearchQuery) -> Result<SearchResults<Node>> {
        let offset = query.offset;
        let limit = query.limit;
        
        // Build the SQL query
        let mut sql = String::from(
            "SELECT id, node_type, properties, created_at, updated_at FROM nodes WHERE 1=1"
        );
        
        // Add node type filters
        if !query.node_types.is_empty() {
            let types: Vec<String> = query.node_types.iter()
                .map(|t| format!("'{}'", t.replace("'", "''")))
                .collect();
            sql.push_str(&format!(" AND node_type IN ({})", types.join(", ")));
        }
        
        // Add text search filter (case-insensitive LIKE on properties)
        if let Some(ref search_text) = query.search_text {
            let escaped = search_text.replace("'", "''").replace("%", "\\%").replace("_", "\\_");
            sql.push_str(&format!(
                " AND properties::text ILIKE '%{}%'",
                escaped
            ));
        }
        
        // Add time range filters
        if let Some(after) = query.created_after {
            sql.push_str(&format!(" AND created_at >= '{}'", after.format("%Y-%m-%d %H:%M:%S")));
        }
        if let Some(before) = query.created_before {
            sql.push_str(&format!(" AND created_at <= '{}'", before.format("%Y-%m-%d %H:%M:%S")));
        }
        if let Some(after) = query.updated_after {
            sql.push_str(&format!(" AND updated_at >= '{}'", after.format("%Y-%m-%d %H:%M:%S")));
        }
        
        // Add property filters
        for (key, value) in &query.property_filters {
            let escaped_key = key.replace("'", "''");
            let escaped_value = value.replace("'", "''");
            sql.push_str(&format!(
                " AND properties->>'{}' = '{}'",
                escaped_key, escaped_value
            ));
        }
        
        // Add ordering by updated_at
        sql.push_str(" ORDER BY updated_at DESC");
        
        // Add limit and offset
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit + 1, offset));
        
        // Execute query
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Parse results
        let mut nodes = Vec::new();
        
        for row in rows.into_iter().take(limit) {
            let properties_json: serde_json::Value = row.try_get("properties")
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let properties: Properties = serde_json::from_value(properties_json)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            nodes.push(Node {
                id: row.try_get("id").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                node_type: row.try_get("node_type").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                properties,
                created_at: row.try_get("created_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
                updated_at: row.try_get("updated_at").map_err(|e| StorageError::DatabaseError(e.to_string()))?,
            });
        }
        
        Ok(SearchResults {
            items: nodes,
        })
    }
}
