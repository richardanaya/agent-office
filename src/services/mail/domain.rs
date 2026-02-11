use crate::domain::{string_to_node_id, Node, NodeId, Properties, PropertyValue, Timestamp};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Domain types for the mail system
pub type MailboxId = NodeId;
pub type MailId = NodeId;
pub type AgentId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mailbox {
    pub id: MailboxId,
    pub owner_id: AgentId,
    pub name: String,
    pub created_at: Timestamp,
}

impl Mailbox {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mail {
    pub id: MailId,
    pub from_mailbox_id: MailboxId,
    pub to_mailbox_id: MailboxId,
    pub subject: String,
    pub body: String,
    pub read: bool,
    pub created_at: Timestamp,
}

impl Mail {
    pub fn new(
        from_mailbox_id: MailboxId,
        to_mailbox_id: MailboxId,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: MailId::new_v4(),
            from_mailbox_id,
            to_mailbox_id,
            subject: subject.into(),
            body: body.into(),
            read: false,
            created_at: Utc::now(),
        }
    }

    pub fn to_node(&self) -> Node {
        let mut props = Properties::new();
        props.insert(
            "from_mailbox_id".to_string(),
            PropertyValue::String(self.from_mailbox_id.to_string()),
        );
        props.insert(
            "to_mailbox_id".to_string(),
            PropertyValue::String(self.to_mailbox_id.to_string()),
        );
        props.insert(
            "subject".to_string(),
            PropertyValue::String(self.subject.clone()),
        );
        props.insert("body".to_string(), PropertyValue::String(self.body.clone()));
        props.insert("read".to_string(), PropertyValue::Boolean(self.read));

        let mut node = Node::new("mail", props);
        node.id = self.id;
        node
    }

    pub fn from_node(node: &Node) -> Option<Self> {
        if node.node_type != "mail" {
            return None;
        }

        let from_mailbox_id = node.get_property("from_mailbox_id").and_then(|v| match v {
            PropertyValue::String(s) => Uuid::parse_str(s).ok(),
            _ => None,
        })?;

        let to_mailbox_id = node.get_property("to_mailbox_id").and_then(|v| match v {
            PropertyValue::String(s) => Uuid::parse_str(s).ok(),
            _ => None,
        })?;

        let subject = node.get_property("subject").and_then(|v| match v {
            PropertyValue::String(s) => Some(s.clone()),
            _ => None,
        })?;

        let body = node.get_property("body").and_then(|v| match v {
            PropertyValue::String(s) => Some(s.clone()),
            _ => None,
        })?;

        let read = node
            .get_property("read")
            .and_then(|v| match v {
                PropertyValue::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

        Some(Self {
            id: node.id,
            from_mailbox_id,
            to_mailbox_id,
            subject,
            body,
            read,
            created_at: node.created_at,
        })
    }

    pub fn mark_as_read(&mut self) {
        self.read = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub status: String,
    pub created_at: Timestamp,
}

impl Default for Agent {
    fn default() -> Self {
        let name = String::from("Unnamed");
        Self {
            id: name.clone(),
            name,
            status: String::from("offline"),
            created_at: Utc::now(),
        }
    }
}

impl Agent {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        // Use the name as the ID for simplicity (allows IDs like "intern_0")
        // Or generate a random string if name is not suitable as ID
        let id = if name.contains(char::is_whitespace) || name.is_empty() {
            format!(
                "agent_{}",
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            )
        } else {
            name.clone()
        };

        Self {
            id,
            name,
            status: String::from("offline"),
            created_at: Utc::now(),
        }
    }

    pub fn to_node(&self) -> Node {
        let mut props = Properties::new();
        props.insert("name".to_string(), PropertyValue::String(self.name.clone()));
        props.insert(
            "agent_id".to_string(),
            PropertyValue::String(self.id.clone()),
        );
        props.insert(
            "status".to_string(),
            PropertyValue::String(self.status.clone()),
        );

        let mut node = Node::new("agent", props);
        // Convert string ID to deterministic UUID for storage
        node.id = string_to_node_id(&self.id);
        node
    }

    pub fn from_node(node: &Node) -> Option<Self> {
        if node.node_type != "agent" {
            return None;
        }

        let name = node.get_property("name").and_then(|v| match v {
            PropertyValue::String(s) => Some(s.clone()),
            _ => None,
        })?;

        // Get agent_id from properties, or convert node.id back to string if not present
        let id = node
            .get_property("agent_id")
            .and_then(|v| match v {
                PropertyValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| node.id.to_string());

        // Get status from properties, default to "offline" if not present
        let status = node
            .get_property("status")
            .and_then(|v| match v {
                PropertyValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| String::from("offline"));

        Some(Self {
            id,
            name,
            status,
            created_at: node.created_at,
        })
    }
}
