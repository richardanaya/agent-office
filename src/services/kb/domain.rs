use crate::domain::{Node, Properties, PropertyValue, Timestamp};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// NoteId is now a LuhmannId - no more UUIDs
pub type NoteId = LuhmannId;

/// Luhmann-style hierarchical ID for Zettelkasten notes
/// Format alternates numbers and letters: 1, 1a, 1a1, 1a1a, 1a2, 1b, 2, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LuhmannId {
    pub parts: Vec<LuhmannPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LuhmannPart {
    Number(u32),
    Letter(char),
}

impl LuhmannId {
    /// Parse a Luhmann ID from string like "1a2b"
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                // Parse number
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(n) = num_str.parse::<u32>() {
                    parts.push(LuhmannPart::Number(n));
                }
            } else if c.is_ascii_alphabetic() {
                // Parse letter (single char)
                parts.push(LuhmannPart::Letter(c.to_ascii_lowercase()));
                chars.next();
            } else {
                chars.next(); // Skip invalid char
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(Self { parts })
        }
    }

    /// Get the parent ID (one level up)
    pub fn parent(&self) -> Option<Self> {
        if self.parts.len() <= 1 {
            None
        } else {
            Some(Self {
                parts: self.parts[..self.parts.len() - 1].to_vec(),
            })
        }
    }

    /// Get the next sibling at the same level
    pub fn next_sibling(&self) -> Option<Self> {
        if let Some(last) = self.parts.last() {
            let mut new_parts = self.parts.clone();
            match last {
                LuhmannPart::Number(n) => {
                    new_parts.pop();
                    new_parts.push(LuhmannPart::Number(n + 1));
                }
                LuhmannPart::Letter(c) => {
                    if let Some(next_char) = (*c as u8 + 1).try_into().ok() {
                        if next_char <= 'z' {
                            new_parts.pop();
                            new_parts.push(LuhmannPart::Letter(next_char));
                        } else {
                            return None; // Can't go past 'z'
                        }
                    }
                }
            }
            Some(Self { parts: new_parts })
        } else {
            None
        }
    }

    /// Get the first child ID (branch off from this note)
    pub fn first_child(&self) -> Self {
        let mut new_parts = self.parts.clone();
        // Alternate: if last was number, add letter; if letter, add number
        match self.parts.last() {
            Some(LuhmannPart::Number(_)) => {
                new_parts.push(LuhmannPart::Letter('a'));
            }
            Some(LuhmannPart::Letter(_)) | None => {
                new_parts.push(LuhmannPart::Number(1));
            }
        }
        Self { parts: new_parts }
    }

    /// Get the level/depth of this ID
    pub fn level(&self) -> usize {
        self.parts.len()
    }

    /// Check if this ID is a descendant of another
    pub fn is_descendant_of(&self, other: &Self) -> bool {
        if other.parts.len() >= self.parts.len() {
            return false;
        }
        self.parts[..other.parts.len()] == other.parts[..]
    }
}

impl std::fmt::Display for LuhmannId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match part {
                LuhmannPart::Number(n) => write!(f, "{}", n)?,
                LuhmannPart::Letter(c) => write!(f, "{}", c)?,
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for LuhmannId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Invalid Luhmann ID: {}", s))
    }
}

/// Simple link type - just "references" with optional context
/// The Luhmann ID provides implicit structure (hierarchy, sequence)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// General reference/link between notes
    References,
}

/// A Zettelkasten-style atomic note
/// The LuhmannId IS the note ID - no UUIDs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: NoteId, // This is now a LuhmannId
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub agent_id: Option<String>, // For jots - associates note with an agent
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Note {
    pub fn new(id: LuhmannId, title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
            agent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_node(&self) -> Node {
        let mut props = Properties::new();
        props.insert(
            "title".to_string(),
            PropertyValue::String(self.title.clone()),
        );
        props.insert(
            "content".to_string(),
            PropertyValue::String(self.content.clone()),
        );
        // Store the Luhmann ID as a property as well for easy lookup
        props.insert(
            "luhmann_id".to_string(),
            PropertyValue::String(self.id.to_string()),
        );
        props.insert(
            "tags".to_string(),
            PropertyValue::List(
                self.tags
                    .iter()
                    .map(|t| PropertyValue::String(t.clone()))
                    .collect(),
            ),
        );
        if let Some(ref agent_id) = self.agent_id {
            props.insert(
                "agent_id".to_string(),
                PropertyValue::String(agent_id.clone()),
            );
        }

        // Convert LuhmannId to a deterministic Node ID string
        let node_id = crate::domain::string_to_node_id(&self.id.to_string());
        let mut node = Node::new("note", props);
        node.id = node_id;
        node.created_at = self.created_at;
        node.updated_at = self.updated_at;
        node
    }

    pub fn from_node(node: &Node) -> Option<Self> {
        if node.node_type != "note" {
            return None;
        }

        let title = node.get_property("title")?.as_str()?.to_string();
        let content = node.get_property("content")?.as_str()?.to_string();

        // Parse LuhmannId from the luhmann_id property
        let luhmann_id = node
            .get_property("luhmann_id")
            .and_then(|v| v.as_str())
            .and_then(|s| LuhmannId::parse(s))?;

        let tags = node
            .get_property("tags")
            .and_then(|v| match v {
                PropertyValue::List(list) => Some(
                    list.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let agent_id = node
            .get_property("agent_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(Self {
            id: luhmann_id,
            title,
            content,
            tags,
            agent_id,
            created_at: node.created_at,
            updated_at: node.updated_at,
        })
    }
}

/// A note link (relationship between two notes)
/// Uses LuhmannIds directly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteLink {
    pub from_note_id: NoteId,
    pub to_note_id: NoteId,
    pub link_type: LinkType,
    pub context: Option<String>,
}

impl NoteLink {
    pub fn new(
        from_note_id: NoteId,
        to_note_id: NoteId,
        link_type: LinkType,
        context: Option<String>,
    ) -> Self {
        Self {
            from_note_id,
            to_note_id,
            link_type,
            context,
        }
    }
}

/// Simple counter for generating next main topic IDs
/// Stored as a special node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteCounter {
    pub next_main_id: u32,
    pub created_at: Timestamp,
}

impl NoteCounter {
    pub fn new() -> Self {
        Self {
            next_main_id: 1,
            created_at: Utc::now(),
        }
    }

    pub fn to_node(&self) -> Node {
        let mut props = Properties::new();
        props.insert(
            "next_main_id".to_string(),
            PropertyValue::Integer(self.next_main_id as i64),
        );

        let mut node = Node::new("note_counter", props);
        // Use a fixed node ID for the counter
        node.id = crate::domain::string_to_node_id("__kb_counter__");
        node.created_at = self.created_at;
        node
    }

    pub fn from_node(node: &Node) -> Option<Self> {
        if node.node_type != "note_counter" {
            return None;
        }

        let next_main_id = node
            .get_property("next_main_id")
            .and_then(|v| match v {
                PropertyValue::Integer(n) => Some(*n as u32),
                _ => Some(1),
            })
            .unwrap_or(1);

        Some(Self {
            next_main_id,
            created_at: node.created_at,
        })
    }

    /// Get and increment the next main topic ID
    pub fn next_main_topic_id(&mut self) -> LuhmannId {
        let id = LuhmannId {
            parts: vec![LuhmannPart::Number(self.next_main_id)],
        };
        self.next_main_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luhmann_id_parsing() {
        let id = LuhmannId::parse("1a2b").unwrap();
        assert_eq!(id.parts.len(), 4);
        assert!(matches!(id.parts[0], LuhmannPart::Number(1)));
        assert!(matches!(id.parts[1], LuhmannPart::Letter('a')));
        assert!(matches!(id.parts[2], LuhmannPart::Number(2)));
        assert!(matches!(id.parts[3], LuhmannPart::Letter('b')));
    }

    #[test]
    fn test_luhmann_id_display() {
        let id = LuhmannId::parse("1a2").unwrap();
        assert_eq!(id.to_string(), "1a2");
    }

    #[test]
    fn test_luhmann_parent() {
        let id = LuhmannId::parse("1a2").unwrap();
        let parent = id.parent().unwrap();
        assert_eq!(parent.to_string(), "1a");
    }

    #[test]
    fn test_luhmann_next_sibling() {
        let id = LuhmannId::parse("1a").unwrap();
        let next = id.next_sibling().unwrap();
        assert_eq!(next.to_string(), "1b");

        let id2 = LuhmannId::parse("1").unwrap();
        let next2 = id2.next_sibling().unwrap();
        assert_eq!(next2.to_string(), "2");
    }

    #[test]
    fn test_luhmann_first_child() {
        let id = LuhmannId::parse("1").unwrap();
        let child = id.first_child();
        assert_eq!(child.to_string(), "1a");

        let id2 = LuhmannId::parse("1a").unwrap();
        let child2 = id2.first_child();
        assert_eq!(child2.to_string(), "1a1");
    }
}
