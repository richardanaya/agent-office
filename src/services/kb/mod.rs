use crate::domain::{Edge, Properties, PropertyValue, string_to_node_id, NodeId};
use crate::services::kb::domain::{LinkType, LuhmannId, Note, NoteId, NoteLink, NoteCounter};
use crate::storage::{GraphStorage, StorageError, SearchQuery, EdgeDirection};
use async_trait::async_trait;
use thiserror::Error;

pub mod domain;

#[derive(Error, Debug)]
pub enum KbError {
    #[error("Note not found: {0}")]
    NoteNotFound(NoteId),
    
    #[error("Note already exists: {0}")]
    NoteAlreadyExists(NoteId),
    
    #[error("Invalid Luhmann ID: {0}")]
    InvalidLuhmannId(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Cannot link note to itself")]
    SelfLink,
}

pub type Result<T> = std::result::Result<T, KbError>;

/// Simplified Knowledge Base Service - shared across all agents, uses only Luhmann IDs
#[async_trait]
pub trait KnowledgeBaseService: Send + Sync {
    // Core note operations (all use LuhmannId)
    async fn create_note(
        &self,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn create_note_with_id(
        &self,
        id: LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn create_branch(
        &self,
        parent_id: &LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note>;
    
    async fn get_note(&self, note_id: &LuhmannId) -> Result<Note>;
    async fn list_notes(&self) -> Result<Vec<Note>>;
    async fn list_notes_by_prefix(&self, prefix: &LuhmannId) -> Result<Vec<Note>>;
    
    // Search
    async fn search_notes(&self, query: &str) -> Result<Vec<Note>>;
    
    // Link operations
    async fn link_notes(
        &self,
        from_id: &LuhmannId,
        to_id: &LuhmannId,
        context: Option<String>,
    ) -> Result<()>;
    
    async fn get_links(&self, note_id: &LuhmannId) -> Result<Vec<NoteLink>>;
    
    // Note relationships
    async fn mark_continuation(&self, from_id: &LuhmannId, to_id: &LuhmannId) -> Result<()>;
    
    // Index operations
    async fn create_index(&self, parent_id: &LuhmannId) -> Result<Note>;
    
    // Get full context of a note
    async fn get_context(&self, note_id: &LuhmannId) -> Result<NoteContext>;
}

/// Full context of a note including all relationships
#[derive(Debug, Clone)]
pub struct NoteContext {
    pub note: Note,
    pub parent: Option<Note>,
    pub children: Vec<Note>,
    pub links_to: Vec<Note>,
    pub backlinks: Vec<Note>,
    pub continues_to: Vec<Note>,
    pub continued_from: Vec<Note>,
}

/// Convert a LuhmannId to a NodeId for storage
fn luhmann_to_node_id(luhmann_id: &LuhmannId) -> NodeId {
    string_to_node_id(&luhmann_id.to_string())
}

pub struct KnowledgeBaseServiceImpl<S: GraphStorage> {
    storage: S,
}

impl<S: GraphStorage> KnowledgeBaseServiceImpl<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Convert LuhmannId to storage NodeId
    fn to_node_id(&self, luhmann_id: &LuhmannId) -> NodeId {
        luhmann_to_node_id(luhmann_id)
    }

    /// Get or initialize the note counter
    async fn get_or_init_counter(&self) -> Result<NoteCounter> {
        let counter_id = string_to_node_id("__kb_counter__");
        match self.storage.get_node(counter_id).await {
            Ok(node) => {
                NoteCounter::from_node(&node)
                    .ok_or_else(|| KbError::Storage(StorageError::ConstraintViolation("Invalid counter node".to_string())))
            }
            Err(StorageError::NodeNotFound(_)) => {
                // Create new counter
                let counter = NoteCounter::new();
                let node = counter.to_node();
                self.storage.create_node(&node).await?;
                Ok(counter)
            }
            Err(e) => Err(KbError::Storage(e)),
        }
    }

    /// Update the counter
    async fn update_counter(&self, counter: &NoteCounter) -> Result<()> {
        let node = counter.to_node();
        self.storage.update_node(&node).await?;
        Ok(())
    }

    /// Generate next available top-level ID
    async fn next_main_id(&self) -> Result<LuhmannId> {
        let mut counter = self.get_or_init_counter().await?;
        let id = counter.next_main_topic_id();
        self.update_counter(&counter).await?;
        Ok(id)
    }

    /// Find the next available child ID under a parent
    async fn next_child_id(&self, parent_id: &LuhmannId) -> Result<LuhmannId> {
        let all_notes = self.list_notes().await?;
        
        // Collect existing children
        let mut children: Vec<LuhmannId> = all_notes
            .into_iter()
            .map(|n| n.id)
            .filter(|id| id.parent().as_ref() == Some(parent_id))
            .collect();
        
        if children.is_empty() {
            // First child
            Ok(parent_id.first_child())
        } else {
            // Find the next sibling after the last child
            children.sort();
            let last = children.last().unwrap();
            Ok(last.next_sibling()
                .unwrap_or_else(|| last.first_child()))
        }
    }
}

#[async_trait]
impl<S: GraphStorage> KnowledgeBaseService for KnowledgeBaseServiceImpl<S> {
    async fn create_note(
        &self,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Generate next available top-level Luhmann ID
        let luhmann_id = self.next_main_id().await?;
        
        // Check if note already exists
        let node_id = self.to_node_id(&luhmann_id);
        match self.storage.get_node(node_id).await {
            Ok(_) => return Err(KbError::NoteAlreadyExists(luhmann_id)),
            Err(StorageError::NodeNotFound(_)) => (), // Good, doesn't exist
            Err(e) => return Err(KbError::Storage(e)),
        }
        
        let note = Note::new(luhmann_id, title, content);
        let node = note.to_node();
        self.storage.create_node(&node).await?;
        
        Ok(note)
    }

    async fn create_note_with_id(
        &self,
        id: LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Check if note already exists
        let node_id = self.to_node_id(&id);
        match self.storage.get_node(node_id).await {
            Ok(_) => return Err(KbError::NoteAlreadyExists(id)),
            Err(StorageError::NodeNotFound(_)) => (), // Good, doesn't exist
            Err(e) => return Err(KbError::Storage(e)),
        }
        
        let note = Note::new(id, title, content);
        let node = note.to_node();
        self.storage.create_node(&node).await?;
        
        Ok(note)
    }

    async fn create_branch(
        &self,
        parent_id: &LuhmannId,
        title: impl Into<String> + Send,
        content: impl Into<String> + Send,
    ) -> Result<Note> {
        // Verify parent exists
        let parent_node_id = self.to_node_id(parent_id);
        self.storage.get_node(parent_node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::NoteNotFound(parent_id.clone()),
                _ => KbError::Storage(e),
            })?;
        
        // Generate child ID
        let child_id = self.next_child_id(parent_id).await?;
        
        // Check if child already exists
        let child_node_id = self.to_node_id(&child_id);
        match self.storage.get_node(child_node_id).await {
            Ok(_) => return Err(KbError::NoteAlreadyExists(child_id)),
            Err(StorageError::NodeNotFound(_)) => (), // Good, doesn't exist
            Err(e) => return Err(KbError::Storage(e)),
        }
        
        // Create the note
        let note = Note::new(child_id.clone(), title, content);
        let node = note.to_node();
        self.storage.create_node(&node).await?;
        
        // Create link to parent
        let mut props = Properties::new();
        props.insert("context".to_string(), PropertyValue::String(format!("Branch of {}", parent_id)));
        
        let edge = Edge::new(
            "references",
            self.to_node_id(&child_id),
            parent_node_id,
            props,
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(note)
    }

    async fn get_note(&self, note_id: &LuhmannId) -> Result<Note> {
        let node_id = self.to_node_id(note_id);
        let node = self.storage.get_node(node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => KbError::NoteNotFound(note_id.clone()),
                _ => KbError::Storage(e),
            })?;
        
        Note::from_node(&node)
            .ok_or_else(|| KbError::NoteNotFound(note_id.clone()))
    }

    async fn list_notes(&self) -> Result<Vec<Note>> {
        // Query all nodes of type "note"
        let query = SearchQuery {
            node_types: vec!["note".to_string()],
            limit: 10000, // Large limit to get all notes
            ..SearchQuery::default()
        };
        
        let results = self.storage.search_nodes(&query).await?;
        
        let mut notes: Vec<Note> = results.items
            .into_iter()
            .filter_map(|node| Note::from_node(&node))
            .collect();
        
        // Sort by LuhmannId for consistent ordering
        notes.sort_by(|a, b| a.id.cmp(&b.id));
        
        Ok(notes)
    }

    async fn list_notes_by_prefix(&self, prefix: &LuhmannId) -> Result<Vec<Note>> {
        let all_notes = self.list_notes().await?;
        
        let filtered: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| {
                note.id == *prefix || note.id.is_descendant_of(prefix)
            })
            .collect();
        
        Ok(filtered)
    }

    async fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_notes().await?;
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

    async fn link_notes(
        &self,
        from_id: &LuhmannId,
        to_id: &LuhmannId,
        context: Option<String>,
    ) -> Result<()> {
        if from_id == to_id {
            return Err(KbError::SelfLink);
        }
        
        // Verify both notes exist
        self.get_note(from_id).await?;
        self.get_note(to_id).await?;
        
        // Create link edge
        let mut props = Properties::new();
        if let Some(ctx) = context {
            props.insert("context".to_string(), PropertyValue::String(ctx));
        }
        
        let edge = Edge::new(
            "references",
            self.to_node_id(from_id),
            self.to_node_id(to_id),
            props,
        );
        
        self.storage.create_edge(&edge).await?;
        Ok(())
    }

    async fn get_links(&self, note_id: &LuhmannId) -> Result<Vec<NoteLink>> {
        let node_id = self.to_node_id(note_id);
        
        // Get outgoing edges
        let edges = self.storage.get_edges_from(node_id, Some("references")).await?;
        
        let mut links = Vec::new();
        for edge in edges {
            // Get the target note by looking up the node and converting it
            match self.storage.get_node(edge.to_node_id).await {
                Ok(target_node) => {
                    if let Some(target_id) = target_node.properties.get("luhmann_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| LuhmannId::parse(s))
                    {
                        let context = edge.properties.get("context")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        
                        links.push(NoteLink::new(
                            note_id.clone(),
                            target_id,
                            LinkType::References,
                            context,
                        ));
                    }
                }
                Err(_) => continue, // Skip notes that can't be found
            }
        }
        
        Ok(links)
    }

    async fn mark_continuation(&self, from_id: &LuhmannId, to_id: &LuhmannId) -> Result<()> {
        if from_id == to_id {
            return Err(KbError::SelfLink);
        }
        
        // Verify both notes exist
        self.get_note(from_id).await?;
        self.get_note(to_id).await?;
        
        // Create "continues" edge
        let mut props = Properties::new();
        props.insert("context".to_string(), PropertyValue::String("Continues on next note".to_string()));
        
        let edge = Edge::new(
            "continues",
            self.to_node_id(from_id),
            self.to_node_id(to_id),
            props,
        );
        
        self.storage.create_edge(&edge).await?;
        Ok(())
    }

    async fn create_index(&self, parent_id: &LuhmannId) -> Result<Note> {
        // Verify parent exists
        let parent_note = self.get_note(parent_id).await?;
        
        // Find all direct children (notes that are immediate descendants)
        let all_notes = self.list_notes().await?;
        
        let children: Vec<&Note> = all_notes
            .iter()
            .filter(|note| {
                // Check if note's parent is the parent_id
                note.id.parent().as_ref() == Some(parent_id)
            })
            .collect();
        
        // Create index note ID: {parent_id}0 (e.g., 1a -> 1a0)
        let index_id = LuhmannId::parse(&format!("{}0", parent_id))
            .ok_or_else(|| KbError::InvalidLuhmannId(format!("{}0", parent_id)))?;
        
        // Check if index already exists
        let index_node_id = self.to_node_id(&index_id);
        match self.storage.get_node(index_node_id).await {
            Ok(_) => return Err(KbError::NoteAlreadyExists(index_id)),
            Err(StorageError::NodeNotFound(_)) => (), // Good, doesn't exist
            Err(e) => return Err(KbError::Storage(e)),
        }
        
        // Build index content
        let mut content = format!("# Index: {}\n\n", parent_note.title);
        content.push_str(&format!("Parent note: [[{}]]\n\n", parent_id));
        content.push_str("Children:\n\n");
        
        if children.is_empty() {
            content.push_str("(No children)\n");
        } else {
            for child in &children {
                content.push_str(&format!("- [[{}]]: {}\n", child.id, child.title));
            }
        }
        
        // Create the index note
        let index_note = Note::new(
            index_id.clone(),
            format!("Index: {}", parent_note.title),
            content,
        );
        
        let node = index_note.to_node();
        self.storage.create_node(&node).await?;
        
        // Create "child_of" relationship to parent
        let mut props = Properties::new();
        props.insert("context".to_string(), PropertyValue::String("Index of children".to_string()));
        
        let edge = Edge::new(
            "child_of",
            index_node_id,
            self.to_node_id(parent_id),
            props,
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(index_note)
    }
    
    async fn get_context(&self, note_id: &LuhmannId) -> Result<NoteContext> {
        // Get the note itself
        let note = self.get_note(note_id).await?;
        
        // Get parent (if any)
        let parent = if let Some(parent_id) = note.id.parent() {
            self.get_note(&parent_id).await.ok()
        } else {
            None
        };
        
        // Get all children (direct descendants)
        let all_notes = self.list_notes().await?;
        let children: Vec<Note> = all_notes
            .into_iter()
            .filter(|n| n.id.parent().as_ref() == Some(note_id))
            .collect();
        
        // Get links (notes this note links TO)
        let links = self.get_links(note_id).await?;
        let mut links_to = Vec::new();
        for link in &links {
            if let Ok(target_note) = self.get_note(&link.to_note_id).await {
                links_to.push(target_note);
            }
        }
        
        // Get backlinks (notes that link TO this note via "references" edges)
        let node_id = self.to_node_id(note_id);
        let edges = self.storage.get_edges_to(node_id, Some("references")).await?;
        let mut backlinks = Vec::new();
        for edge in edges {
            if let Ok(source_node) = self.storage.get_node(edge.from_node_id).await {
                if let Some(note) = Note::from_node(&source_node) {
                    backlinks.push(note);
                }
            }
        }
        
        // Get continuations (notes this note "continues" to)
        // Need to query for "continues" edges
        let note_node_id = self.to_node_id(note_id);
        let neighbors = self.storage
            .get_neighbors(note_node_id, Some("continues"), EdgeDirection::Outgoing)
            .await?;
        
        let mut continues_to = Vec::new();
        for node in neighbors {
            if let Some(luhmann_str) = node.get_property("luhmann_id").and_then(|v| v.as_str()) {
                if let Some(target_id) = LuhmannId::parse(luhmann_str) {
                    if let Ok(target_note) = self.get_note(&target_id).await {
                        continues_to.push(target_note);
                    }
                }
            }
        }
        
        // Get notes that continue FROM this note (reverse of continues)
        let incoming_neighbors = self.storage
            .get_neighbors(note_node_id, Some("continues"), EdgeDirection::Incoming)
            .await?;
        
        let mut continued_from = Vec::new();
        for node in incoming_neighbors {
            if let Some(luhmann_str) = node.get_property("luhmann_id").and_then(|v| v.as_str()) {
                if let Some(source_id) = LuhmannId::parse(luhmann_str) {
                    if let Ok(source_note) = self.get_note(&source_id).await {
                        continued_from.push(source_note);
                    }
                }
            }
        }
        
        Ok(NoteContext {
            note,
            parent,
            children,
            links_to,
            backlinks,
            continues_to,
            continued_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::InMemoryStorage;

    #[tokio::test]
    async fn test_create_note_auto_id() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        // First note should get ID "1"
        let note1 = kb.create_note("First Note", "Content 1").await.unwrap();
        assert_eq!(note1.id.to_string(), "1");
        
        // Second note should get ID "2"
        let note2 = kb.create_note("Second Note", "Content 2").await.unwrap();
        assert_eq!(note2.id.to_string(), "2");
    }

    #[tokio::test]
    async fn test_create_note_with_specific_id() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let id = LuhmannId::parse("1a").unwrap();
        let note = kb.create_note_with_id(id.clone(), "Note 1a", "Content").await.unwrap();
        assert_eq!(note.id, id);
    }

    #[tokio::test]
    async fn test_create_branch() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        // Create parent note "1"
        let parent_id = LuhmannId::parse("1").unwrap();
        kb.create_note_with_id(parent_id.clone(), "Parent", "Parent content").await.unwrap();
        
        // Create branch - should get ID "1a"
        let child = kb.create_branch(&parent_id, "Child", "Child content").await.unwrap();
        assert_eq!(child.id.to_string(), "1a");
        
        // Create another branch - should get ID "1b"
        let child2 = kb.create_branch(&parent_id, "Child 2", "Child content 2").await.unwrap();
        assert_eq!(child2.id.to_string(), "1b");
    }

    #[tokio::test]
    async fn test_duplicate_id_fails() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let id = LuhmannId::parse("1").unwrap();
        kb.create_note_with_id(id.clone(), "First", "Content").await.unwrap();
        
        // Creating with same ID should fail
        let result = kb.create_note_with_id(id, "Second", "Content").await;
        assert!(matches!(result, Err(KbError::NoteAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_get_note() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let note = kb.create_note("Test", "Content").await.unwrap();
        let retrieved = kb.get_note(&note.id).await.unwrap();
        
        assert_eq!(retrieved.title, "Test");
        assert_eq!(retrieved.content, "Content");
    }

    #[tokio::test]
    async fn test_list_notes() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        kb.create_note_with_id(LuhmannId::parse("2").unwrap(), "Second", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("1").unwrap(), "First", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("1a").unwrap(), "Child", "Content").await.unwrap();
        
        let notes = kb.list_notes().await.unwrap();
        
        // Should be sorted by Luhmann ID
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].id.to_string(), "1");
        assert_eq!(notes[1].id.to_string(), "1a");
        assert_eq!(notes[2].id.to_string(), "2");
    }

    #[tokio::test]
    async fn test_list_by_prefix() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        kb.create_note_with_id(LuhmannId::parse("1").unwrap(), "One", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("1a").unwrap(), "One-A", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("1a1").unwrap(), "One-A-One", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("1b").unwrap(), "One-B", "Content").await.unwrap();
        kb.create_note_with_id(LuhmannId::parse("2").unwrap(), "Two", "Content").await.unwrap();
        
        let prefix = LuhmannId::parse("1a").unwrap();
        let notes = kb.list_notes_by_prefix(&prefix).await.unwrap();
        
        assert_eq!(notes.len(), 2); // 1a and 1a1
        assert!(notes.iter().any(|n| n.id.to_string() == "1a"));
        assert!(notes.iter().any(|n| n.id.to_string() == "1a1"));
    }

    #[tokio::test]
    async fn test_search_notes() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        kb.create_note("Rust Programming", "A systems language").await.unwrap();
        kb.create_note("Python Basics", "Easy to learn").await.unwrap();
        kb.create_note("Rust vs Go", "Comparison").await.unwrap();
        
        let results = kb.search_notes("rust").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_link_notes() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let note1 = kb.create_note("First", "Content").await.unwrap();
        let note2 = kb.create_note("Second", "Content").await.unwrap();
        
        kb.link_notes(&note1.id, &note2.id, Some("See also".to_string())).await.unwrap();
        
        let links = kb.get_links(&note1.id).await.unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].to_note_id, note2.id);
    }

    #[tokio::test]
    async fn test_self_link_fails() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let note = kb.create_note("Note", "Content").await.unwrap();
        
        let result = kb.link_notes(&note.id, &note.id, None).await;
        assert!(matches!(result, Err(KbError::SelfLink)));
    }

    #[tokio::test]
    async fn test_mark_continuation() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        let id1 = LuhmannId::parse("1").unwrap();
        let id2 = LuhmannId::parse("2").unwrap();
        
        kb.create_note_with_id(id1.clone(), "First", "Content 1").await.unwrap();
        kb.create_note_with_id(id2.clone(), "Second", "Content 2").await.unwrap();
        
        // Mark note 1 as continuing to note 2
        kb.mark_continuation(&id1, &id2).await.unwrap();
        
        // Self-continuation should fail
        let result = kb.mark_continuation(&id1, &id1).await;
        assert!(matches!(result, Err(KbError::SelfLink)));
    }

    #[tokio::test]
    async fn test_create_index() {
        let storage = InMemoryStorage::new();
        let kb = KnowledgeBaseServiceImpl::new(storage);
        
        // Create parent and children
        let parent_id = LuhmannId::parse("1").unwrap();
        kb.create_note_with_id(parent_id.clone(), "Parent Note", "Parent content").await.unwrap();
        
        let child1_id = LuhmannId::parse("1a").unwrap();
        kb.create_note_with_id(child1_id.clone(), "First Child", "Child 1 content").await.unwrap();
        
        let child2_id = LuhmannId::parse("1b").unwrap();
        kb.create_note_with_id(child2_id.clone(), "Second Child", "Child 2 content").await.unwrap();
        
        // Create grandchild (should not appear in index)
        let grandchild_id = LuhmannId::parse("1a1").unwrap();
        kb.create_note_with_id(grandchild_id.clone(), "Grandchild", "Grandchild content").await.unwrap();
        
        // Create index
        let index = kb.create_index(&parent_id).await.unwrap();
        
        // Index ID should be {parent_id}0
        assert_eq!(index.id.to_string(), "10");
        // Should contain references to children
        assert!(index.content.contains("1a"));
        assert!(index.content.contains("1b"));
        assert!(index.content.contains("First Child"));
        assert!(index.content.contains("Second Child"));
        // Should NOT contain grandchild
        assert!(!index.content.contains("1a1"));
    }
}
