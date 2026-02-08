use crate::domain::{Edge, Properties, PropertyValue, string_to_node_id};
use crate::services::kb::domain::{KnowledgeBase, LinkType, LuhmannId, Note, NoteId, NoteLink, AgentId};
use crate::storage::{GraphStorage, StorageError, EdgeDirection};
use async_trait::async_trait;
use thiserror::Error;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod domain;

#[derive(Error, Debug)]
pub enum KbError {
    #[error("Note not found: {0}")]
    NoteNotFound(NoteId),
    
    #[error("Knowledge base not found for agent: {0}")]
    KnowledgeBaseNotFound(AgentId),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),
    
    #[error("Invalid link type: {0}")]
    InvalidLinkType(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Note already linked")]
    AlreadyLinked,
    
    #[error("Cannot link note to itself")]
    SelfLink,
}

pub type Result<T> = std::result::Result<T, KbError>;

#[async_trait]
pub trait KnowledgeBaseService: Send + Sync {
    // Knowledge Base operations
    async fn create_knowledge_base(&self, agent_id: AgentId, name: impl Into<String> + Send) -> Result<KnowledgeBase>;
    async fn get_knowledge_base(&self, agent_id: AgentId) -> Result<KnowledgeBase>;
    
    // Note operations
    async fn create_note(
        &self,
        agent_id: AgentId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn get_note(&self, note_id: NoteId) -> Result<Note>;
    async fn update_note(&self, note: &Note) -> Result<Note>;
    async fn delete_note(&self, note_id: NoteId) -> Result<()>;
    async fn list_agent_notes(&self, agent_id: AgentId) -> Result<Vec<Note>>;
    async fn search_notes(&self, agent_id: AgentId, query: &str) -> Result<Vec<Note>>;
    
    // Tag operations
    async fn add_tag(&self, note_id: NoteId, tag: impl Into<String> + Send) -> Result<Note>;
    async fn remove_tag(&self, note_id: NoteId, tag: &str) -> Result<Note>;
    async fn list_notes_by_tag(&self, agent_id: AgentId, tag: &str) -> Result<Vec<Note>>;
    async fn get_all_tags(&self, agent_id: AgentId) -> Result<Vec<String>>;
    
    // Link operations
    async fn link_notes(
        &self,
        from_note_id: NoteId,
        to_note_id: NoteId,
        link_type: LinkType,
        context: Option<String>,
    ) -> Result<()>;
    
    async fn unlink_notes(&self, from_note_id: NoteId, to_note_id: NoteId, link_type: LinkType) -> Result<()>;
    async fn get_links_from(&self, note_id: NoteId, link_type: Option<LinkType>) -> Result<Vec<NoteLink>>;
    async fn get_links_to(&self, note_id: NoteId, link_type: Option<LinkType>) -> Result<Vec<NoteLink>>;
    async fn get_backlinks(&self, note_id: NoteId) -> Result<Vec<Note>>;
    
    // Graph traversal
    async fn get_related_notes(&self, note_id: NoteId, depth: usize) -> Result<Vec<Note>>;
    async fn find_path(&self, start_note_id: NoteId, end_note_id: NoteId, max_depth: usize) -> Result<Option<Vec<NoteId>>>;
    async fn get_note_graph(&self, note_id: NoteId, depth: usize) -> Result<NoteGraph>;
    
    // Luhmann ID operations (Zettelkasten addressing)
    async fn create_note_with_luhmann_id(
        &self,
        agent_id: AgentId,
        luhmann_id: LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn create_note_branch(
        &self,
        agent_id: AgentId,
        parent_note_id: NoteId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn get_note_by_luhmann_id(&self, agent_id: AgentId, luhmann_id: &LuhmannId) -> Result<Option<Note>>;
    async fn get_next_available_id(&self, agent_id: AgentId, parent_id: Option<&LuhmannId>) -> Result<LuhmannId>;
    async fn list_notes_by_luhmann_prefix(&self, agent_id: AgentId, prefix: &LuhmannId) -> Result<Vec<Note>>;
}

/// Represents a subgraph of related notes
#[derive(Debug, Clone)]
pub struct NoteGraph {
    pub center_note_id: NoteId,
    pub notes: Vec<Note>,
    pub links: Vec<NoteLink>,
}

pub struct KnowledgeBaseServiceImpl<S: GraphStorage> {
    storage: S,
}

impl<S: GraphStorage> KnowledgeBaseServiceImpl<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    async fn note_exists(&self, note_id: NoteId) -> Result<bool> {
        match self.storage.get_node(note_id).await {
            Ok(node) => Ok(Note::from_node(&node).is_some()),
            Err(StorageError::NodeNotFound(_)) => Ok(false),
            Err(e) => Err(KbError::Storage(e)),
        }
    }
}

#[async_trait]
impl<S: GraphStorage> KnowledgeBaseService for KnowledgeBaseServiceImpl<S> {
    async fn create_knowledge_base(&self, agent_id: AgentId, name: impl Into<String> + Send) -> Result<KnowledgeBase> {
        // Clone agent_id for error handling
        let agent_id_for_err = agent_id.clone();
        // Get agent and verify it exists
        let node_id = string_to_node_id(&agent_id);
        let node = self.storage.get_node(node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::AgentNotFound(agent_id_for_err),
                _ => KbError::Storage(e),
            })?;
        
        // Agent node becomes the knowledge base - update its properties
        let mut kb = KnowledgeBase::new(agent_id.clone(), name);
        kb.agent_id = agent_id.clone(); // Already set but explicit
        
        // Update the agent node with kb metadata
        let mut updated_node = node.clone();
        updated_node.properties.insert("kb_name".to_string(), PropertyValue::String(kb.name.clone()));
        updated_node.properties.insert("kb_enabled".to_string(), PropertyValue::Boolean(true));
        updated_node.properties.insert("next_main_id".to_string(), PropertyValue::Integer(1));
        
        self.storage.update_node(&updated_node).await?;
        Ok(kb)
    }

    async fn get_knowledge_base(&self, agent_id: AgentId) -> Result<KnowledgeBase> {
        let agent_id_err = agent_id.clone();
        let node_id = string_to_node_id(&agent_id);
        let node = self.storage.get_node(node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::KnowledgeBaseNotFound(agent_id_err),
                _ => KbError::Storage(e),
            })?;
        
        // Check if kb_enabled is set
        let kb_enabled = node.get_property("kb_enabled")
            .and_then(|v| match v {
                PropertyValue::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);
        
        if !kb_enabled {
            return Err(KbError::KnowledgeBaseNotFound(agent_id.clone()));
        }
        
        let name = node.get_property("kb_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled KB")
            .to_string();
        
        let next_main_id = node.get_property("next_main_id")
            .and_then(|v| match v {
                PropertyValue::Integer(n) => Some(*n as u32),
                _ => Some(1),
            })
            .unwrap_or(1);
        
        // Get agent_id from node properties or use a default
        let agent_id = node.get_property("agent_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| node.id.to_string());
        
        Ok(KnowledgeBase {
            agent_id,
            name,
            description: None,
            created_at: node.created_at,
            next_main_id,
        })
    }

    async fn create_note(
        &self,
        agent_id: AgentId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Clone agent_id for multiple uses
        let agent_id_for_err = agent_id.clone();
        
        // Verify agent/knowledge base exists
        let _ = self.storage.get_node(string_to_node_id(&agent_id)).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::AgentNotFound(agent_id_for_err),
                _ => KbError::Storage(e),
            })?;
        
        // Generate next available Luhmann ID for top-level note
        let luhmann_id = self.get_next_available_id(agent_id.clone(), None).await?;
        
        let note = Note::new(agent_id.clone(), title, content)
            .with_luhmann_id(luhmann_id);
        let node = note.to_node();
        self.storage.create_node(&node).await?;
        
        // Create ownership edge from agent to note
        let edge = Edge::new(
            "owns_note",
            string_to_node_id(&agent_id),
            note.id,
            Properties::new(),
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(note)
    }

    async fn get_note(&self, note_id: NoteId) -> Result<Note> {
        let node = self.storage.get_node(note_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::NoteNotFound(note_id),
                _ => KbError::Storage(e),
            })?;
        
        Note::from_node(&node)
            .ok_or_else(|| KbError::NoteNotFound(note_id))
    }

    async fn update_note(&self, note: &Note) -> Result<Note> {
        // Verify note exists
        self.get_note(note.id).await?;
        
        let node = note.to_node();
        self.storage.update_node(&node).await?;
        Ok(note.clone())
    }

    async fn delete_note(&self, note_id: NoteId) -> Result<()> {
        // Verify note exists
        self.get_note(note_id).await?;
        
        // Delete the note (edges will be cascade deleted)
        self.storage.delete_node(note_id).await?;
        Ok(())
    }

    async fn list_agent_notes(&self, agent_id: AgentId) -> Result<Vec<Note>> {
        // Clone for error handling
        let agent_id_err = agent_id.clone();
        
        // Verify agent exists
        let agent_node_id = string_to_node_id(&agent_id);
        let _ = self.storage.get_node(agent_node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::AgentNotFound(agent_id_err),
                _ => KbError::Storage(e),
            })?;
        
        // Get all notes owned by this agent
        let notes = self.storage
            .get_neighbors(agent_node_id, Some("owns_note"), EdgeDirection::Outgoing)
            .await?;
        
        let notes: Vec<Note> = notes.iter()
            .filter_map(Note::from_node)
            .collect();
        
        Ok(notes)
    }

    async fn search_notes(&self, agent_id: AgentId, query: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_agent_notes(agent_id).await?;
        let query_lower = query.to_lowercase();
        
        let filtered: Vec<Note> = all_notes.into_iter()
            .filter(|note| {
                note.title.to_lowercase().contains(&query_lower) ||
                note.content.to_lowercase().contains(&query_lower) ||
                note.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();
        
        Ok(filtered)
    }

    async fn add_tag(&self, note_id: NoteId, tag: impl Into<String> + Send) -> Result<Note> {
        let mut note = self.get_note(note_id).await?;
        note.add_tag(tag);
        self.update_note(&note).await?;
        Ok(note)
    }

    async fn remove_tag(&self, note_id: NoteId, tag: &str) -> Result<Note> {
        let mut note = self.get_note(note_id).await?;
        note.remove_tag(tag);
        self.update_note(&note).await?;
        Ok(note)
    }

    async fn list_notes_by_tag(&self, agent_id: AgentId, tag: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_agent_notes(agent_id).await?;
        let filtered: Vec<Note> = all_notes.into_iter()
            .filter(|note| note.tags.contains(&tag.to_string()))
            .collect();
        Ok(filtered)
    }

    async fn get_all_tags(&self, agent_id: AgentId) -> Result<Vec<String>> {
        let notes = self.list_agent_notes(agent_id).await?;
        let mut tags = HashSet::new();
        for note in notes {
            for tag in note.tags {
                tags.insert(tag);
            }
        }
        let mut tags: Vec<String> = tags.into_iter().collect();
        tags.sort();
        Ok(tags)
    }

    async fn link_notes(
        &self,
        from_note_id: NoteId,
        to_note_id: NoteId,
        link_type: LinkType,
        context: Option<String>,
    ) -> Result<()> {
        if from_note_id == to_note_id {
            return Err(KbError::SelfLink);
        }
        
        // Verify both notes exist
        self.get_note(from_note_id).await?;
        self.get_note(to_note_id).await?;
        
        // Create link edge with context in properties
        let mut props = Properties::new();
        if let Some(ctx) = context {
            props.insert("context".to_string(), crate::domain::PropertyValue::String(ctx));
        }
        
        let edge = Edge::new(
            link_type.as_str(),
            from_note_id,
            to_note_id,
            props,
        );
        
        self.storage.create_edge(&edge).await?;
        Ok(())
    }

    async fn unlink_notes(&self, from_note_id: NoteId, to_note_id: NoteId, link_type: LinkType) -> Result<()> {
        // Get edges from the source note
        let edges = self.storage.get_edges_from(from_note_id, Some(link_type.as_str())).await?;
        
        // Find and delete the specific edge
        for edge in edges {
            if edge.to_node_id == to_note_id {
                self.storage.delete_edge(edge.id).await?;
                return Ok(());
            }
        }
        
        Err(KbError::NoteNotFound(to_note_id))
    }

    async fn get_links_from(&self, note_id: NoteId, link_type: Option<LinkType>) -> Result<Vec<NoteLink>> {
        let edges = if let Some(lt) = link_type {
            self.storage.get_edges_from(note_id, Some(lt.as_str())).await?
        } else {
            self.storage.get_edges_from(note_id, None).await?
        };
        
        let links: Vec<NoteLink> = edges.iter()
            .filter_map(|edge| {
                LinkType::from_str(&edge.edge_type).map(|lt| {
                    let context = edge.properties.get("context")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    
                    NoteLink::new(edge.from_node_id, edge.to_node_id, lt, context)
                })
            })
            .collect();
        
        Ok(links)
    }

    async fn get_links_to(&self, note_id: NoteId, link_type: Option<LinkType>) -> Result<Vec<NoteLink>> {
        let edges = if let Some(lt) = link_type {
            self.storage.get_edges_to(note_id, Some(lt.as_str())).await?
        } else {
            self.storage.get_edges_to(note_id, None).await?
        };
        
        let links: Vec<NoteLink> = edges.iter()
            .filter_map(|edge| {
                LinkType::from_str(&edge.edge_type).map(|lt| {
                    let context = edge.properties.get("context")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    
                    NoteLink::new(edge.from_node_id, edge.to_node_id, lt, context)
                })
            })
            .collect();
        
        Ok(links)
    }

    async fn get_backlinks(&self, note_id: NoteId) -> Result<Vec<Note>> {
        // Get all edges pointing to this note
        let edges = self.storage.get_edges_to(note_id, None).await?;
        
        let mut notes = Vec::new();
        for edge in edges {
            if let Ok(note) = self.get_note(edge.from_node_id).await {
                notes.push(note);
            }
        }
        
        Ok(notes)
    }

    async fn get_related_notes(&self, note_id: NoteId, depth: usize) -> Result<Vec<Note>> {
        if depth == 0 {
            return Ok(vec![]);
        }
        
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        
        queue.push_back((note_id, 0usize));
        visited.insert(note_id);
        
        while let Some((current_id, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }
            
            // Get all neighbors (both outgoing and incoming edges)
            let neighbors = self.storage
                .get_neighbors(current_id, None, EdgeDirection::Both)
                .await?;
            
            for neighbor in neighbors {
                if visited.insert(neighbor.id) {
                    if let Some(note) = Note::from_node(&neighbor) {
                        result.push(note);
                        queue.push_back((neighbor.id, current_depth + 1));
                    }
                }
            }
        }
        
        Ok(result)
    }

    async fn find_path(&self, start_note_id: NoteId, end_note_id: NoteId, max_depth: usize) -> Result<Option<Vec<NoteId>>> {
        if start_note_id == end_note_id {
            return Ok(Some(vec![start_note_id]));
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<NoteId, NoteId> = HashMap::new();
        
        queue.push_back((start_note_id, 0usize));
        visited.insert(start_note_id);
        
        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            // Get neighbors (outgoing only for path finding)
            let neighbors = self.storage
                .get_neighbors(current_id, None, EdgeDirection::Outgoing)
                .await?;
            
            for neighbor in neighbors {
                if visited.insert(neighbor.id) {
                    parent_map.insert(neighbor.id, current_id);
                    
                    if neighbor.id == end_note_id {
                        // Reconstruct path
                        let mut path = vec![end_note_id];
                        let mut current = end_note_id;
                        
                        while let Some(&parent) = parent_map.get(&current) {
                            path.push(parent);
                            current = parent;
                        }
                        
                        path.reverse();
                        return Ok(Some(path));
                    }
                    
                    queue.push_back((neighbor.id, depth + 1));
                }
            }
        }
        
        Ok(None)
    }

    async fn get_note_graph(&self, note_id: NoteId, depth: usize) -> Result<NoteGraph> {
        let center_note = self.get_note(note_id).await?;
        let related_notes = self.get_related_notes(note_id, depth).await?;
        
        let mut notes = vec![center_note.clone()];
        notes.extend(related_notes);
        
        // Collect all links between notes in the graph
        let mut links = Vec::new();
        let note_ids: HashSet<NoteId> = notes.iter().map(|n| n.id).collect();
        
        for note in &notes {
            let outgoing = self.get_links_from(note.id, None).await?;
            for link in outgoing {
                if note_ids.contains(&link.to_note_id) {
                    links.push(link);
                }
            }
        }
        
        Ok(NoteGraph {
            center_note_id: note_id,
            notes,
            links,
        })
    }

    async fn create_note_with_luhmann_id(
        &self,
        agent_id: AgentId,
        luhmann_id: LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Clone agent_id for multiple uses
        let agent_id_for_err = agent_id.clone();
        
        // Verify agent/knowledge base exists
        let _ = self.storage.get_node(string_to_node_id(&agent_id)).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::AgentNotFound(agent_id_for_err),
                _ => KbError::Storage(e),
            })?;
        
        let note = Note::new(agent_id.clone(), title, content)
            .with_luhmann_id(luhmann_id);
        let node = note.to_node();
        self.storage.create_node(&node).await?;
        
        // Create ownership edge from agent to note
        let edge = Edge::new(
            "owns_note",
            string_to_node_id(&agent_id),
            note.id,
            Properties::new(),
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(note)
    }

    async fn create_note_branch(
        &self,
        agent_id: AgentId,
        parent_note_id: NoteId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Get parent note to find its Luhmann ID
        let parent_note = self.get_note(parent_note_id).await?;
        
        let parent_luhmann_id = parent_note.luhmann_id
            .ok_or_else(|| KbError::NoteNotFound(parent_note_id))?;
        
        // Generate the next available child ID
        let child_id = self.get_next_available_id(agent_id.clone(), Some(&parent_luhmann_id)).await?;
        
        // Create the note with the Luhmann ID
        let note = self.create_note_with_luhmann_id(
            agent_id,
            child_id.clone(),
            title,
            content,
        ).await?;
        
        // Create reference link to parent
        self.link_notes(note.id, parent_note_id, LinkType::References, Some(format!("Branch of {}", parent_luhmann_id))).await?;
        
        Ok(note)
    }

    async fn get_note_by_luhmann_id(&self, agent_id: AgentId, luhmann_id: &LuhmannId) -> Result<Option<Note>> {
        // Get all notes for this agent and find the one with matching Luhmann ID
        let notes = self.list_agent_notes(agent_id).await?;
        
        Ok(notes.into_iter()
            .find(|note| note.luhmann_id.as_ref() == Some(luhmann_id)))
    }

    async fn get_next_available_id(&self, agent_id: AgentId, parent_id: Option<&LuhmannId>) -> Result<LuhmannId> {
        let all_notes = self.list_agent_notes(agent_id).await?;
        
        // Collect all existing Luhmann IDs at the specified level
        let existing_ids: Vec<LuhmannId> = all_notes
            .iter()
            .filter_map(|note| note.luhmann_id.clone())
            .filter(|id| {
                if let Some(parent) = parent_id {
                    // Check if this ID is a direct child of the parent
                    id.parent().as_ref() == Some(parent)
                } else {
                    // Top-level IDs have no parent
                    id.parent().is_none()
                }
            })
            .collect();
        
        if let Some(parent) = parent_id {
            // Generate next sibling under parent
            if existing_ids.is_empty() {
                // First child of parent
                Ok(parent.first_child())
            } else {
                // Find the last child and get next sibling
                let mut sorted = existing_ids.clone();
                sorted.sort();
                if let Some(last) = sorted.last() {
                    Ok(last.next_sibling()
                        .unwrap_or_else(|| last.first_child()))
                } else {
                    Ok(parent.first_child())
                }
            }
        } else {
            // Top-level - generate next main topic ID
            if existing_ids.is_empty() {
                Ok(LuhmannId { parts: vec![crate::services::kb::domain::LuhmannPart::Number(1)] })
            } else {
                let mut sorted = existing_ids;
                sorted.sort();
                if let Some(last) = sorted.last() {
                    Ok(last.next_sibling()
                        .unwrap_or_else(|| LuhmannId { parts: vec![crate::services::kb::domain::LuhmannPart::Number(1)] }))
                } else {
                    Ok(LuhmannId { parts: vec![crate::services::kb::domain::LuhmannPart::Number(1)] })
                }
            }
        }
    }

    async fn list_notes_by_luhmann_prefix(&self, agent_id: AgentId, prefix: &LuhmannId) -> Result<Vec<Note>> {
        let all_notes = self.list_agent_notes(agent_id).await?;
        
        // Filter notes whose Luhmann ID starts with the prefix
        let filtered: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| {
                if let Some(ref lid) = note.luhmann_id {
                    lid.is_descendant_of(prefix) || lid == prefix
                } else {
                    false
                }
            })
            .collect();
        
        Ok(filtered)
    }
}
