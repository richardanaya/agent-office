use crate::domain::{Edge, GraphQuery, Properties, string_to_node_id};
use crate::services::mail::domain::{Agent, AgentId, Mail, Mailbox, MailboxId};
use crate::storage::{GraphStorage, StorageError};
use async_trait::async_trait;
use thiserror::Error;

pub mod domain;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("Mailbox not found: {0}")]
    MailboxNotFound(MailboxId),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),
    
    #[error("Mail not found: {0}")]
    MailNotFound(uuid::Uuid),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Invalid agent name: {0}")]
    InvalidAgentName(String),
}

pub type Result<T> = std::result::Result<T, MailError>;

#[async_trait]
pub trait MailService: Send + Sync {
    // Agent operations
    async fn create_agent(&self, name: impl Into<String> + Send) -> Result<Agent>;
    async fn delete_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn get_agent(&self, id: AgentId) -> Result<Agent>;
    async fn list_agents(&self) -> Result<Vec<Agent>>;
    async fn set_agent_status(&self, agent_id: AgentId, status: impl Into<String> + Send) -> Result<Agent>;
    
    // Get agent by their mailbox ID (each agent has exactly one mailbox)
    async fn get_agent_by_mailbox(&self, mailbox_id: MailboxId) -> Result<Agent>;
    
    // Get the single mailbox for an agent (auto-creates if doesn't exist)
    async fn get_agent_mailbox(&self, agent_id: AgentId) -> Result<Mailbox>;
    
    // Send mail from one agent to another
    async fn send_agent_to_agent(
        &self,
        from_agent_id: AgentId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail>;
    
    // Get mail received by an agent's mailbox
    async fn get_mailbox_inbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>>;
    
    // Get mail sent by an agent's mailbox
    async fn get_mailbox_outbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>>;
    
    // Get recent mail for an agent (received in last N hours)
    async fn get_recent_mail(&self, mailbox_id: MailboxId, hours: i64, limit: usize) -> Result<Vec<Mail>>;
    
    // Mark mail as read
    async fn mark_mail_as_read(&self, mail_id: uuid::Uuid) -> Result<Mail>;
    
    // Mark mail as read by short ID (8-char prefix) - searches all mail system-wide
    async fn mark_mail_as_read_by_short_id(&self, short_id: &str) -> Result<Mail>;
    
    // Check if agent has unread mail
    async fn check_unread_mail(&self, agent_id: AgentId) -> Result<(bool, Vec<Mail>)>;
}

pub struct MailServiceImpl<S: GraphStorage> {
    storage: S,
}

impl<S: GraphStorage> MailServiceImpl<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Helper to get mail by ID
    async fn get_mail(&self, mail_id: uuid::Uuid) -> Result<Mail> {
        let node = self.storage.get_node(mail_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailNotFound(mail_id),
                _ => MailError::Storage(e),
            })?;
        Mail::from_node(&node)
            .ok_or(MailError::MailNotFound(mail_id))
    }
}

#[async_trait]
impl<S: GraphStorage> MailService for MailServiceImpl<S> {
    async fn create_agent(&self, name: impl Into<String> + Send) -> Result<Agent> {
        let name = name.into();
        
        // Validate agent name: must be lowercase with no spaces
        if name.chars().any(|c| c.is_uppercase() || c.is_whitespace()) {
            return Err(MailError::InvalidAgentName(
                format!("Agent name '{}' is invalid. Names must be lowercase with no spaces (e.g., 'my_agent', 'agent123').", name)
            ));
        }
        
        let agent = Agent::new(name);
        let node = agent.to_node();
        self.storage.create_node(&node).await?;
        
        Ok(agent)
    }

    async fn delete_agent(&self, agent_id: AgentId) -> Result<()> {
        // First, get the agent to ensure they exist
        let agent = self.get_agent(agent_id.clone()).await?;
        let agent_node_id = string_to_node_id(&agent.id);
        
        // Get all mail in inbox and outbox to clear them
        let inbox = self.get_mailbox_inbox(agent_node_id).await?;
        let outbox = self.get_mailbox_outbox(agent_node_id).await?;
        
        // Delete all mail nodes
        for mail in inbox.iter().chain(outbox.iter()) {
            let _ = self.storage.delete_node(mail.id).await;
        }
        
        // Finally, delete the agent node
        self.storage.delete_node(agent_node_id).await?;
        
        Ok(())
    }

    async fn get_agent(&self, id: AgentId) -> Result<Agent> {
        let node_id = string_to_node_id(&id);
        let id_clone = id.clone();
        let node = self.storage.get_node(node_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::AgentNotFound(id_clone),
                _ => MailError::Storage(e),
            })?;
        Agent::from_node(&node)
            .ok_or(MailError::AgentNotFound(id))
    }

    async fn list_agents(&self) -> Result<Vec<Agent>> {
        let query = GraphQuery::new().with_node_type("agent");
        let nodes = self.storage.query_nodes(&query).await?;
        let agents: Vec<Agent> = nodes.iter()
            .filter_map(Agent::from_node)
            .collect();
        Ok(agents)
    }

    async fn set_agent_status(&self, agent_id: AgentId, status: impl Into<String> + Send) -> Result<Agent> {
        let mut agent = self.get_agent(agent_id).await?;
        agent.status = status.into();
        let node = agent.to_node();
        self.storage.update_node(&node).await?;
        Ok(agent)
    }

    async fn get_agent_by_mailbox(&self, mailbox_id: MailboxId) -> Result<Agent> {
        // The mailbox ID is the agent's node ID, so get the agent directly
        let node = self.storage.get_node(mailbox_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailboxNotFound(mailbox_id),
                _ => MailError::Storage(e),
            })?;
        
        Agent::from_node(&node)
            .ok_or_else(|| MailError::InvalidOperation(
                "Node exists but is not an agent".to_string()
            ))
    }

    async fn get_agent_mailbox(&self, agent_id: AgentId) -> Result<Mailbox> {
        // Verify agent exists - the agent's node IS the mailbox
        let agent = self.get_agent(agent_id.clone()).await?;
        let node_id = string_to_node_id(&agent.id);
        
        // Create a mailbox representation from the agent
        let mailbox = Mailbox {
            id: node_id,
            owner_id: agent.id,
            name: "Mailbox".to_string(),
            created_at: agent.created_at,
        };
        
        Ok(mailbox)
    }

    async fn send_agent_to_agent(
        &self,
        from_agent_id: AgentId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail> {
        // Verify both agents exist
        let from_agent = self.get_agent(from_agent_id).await?;
        let to_agent = self.get_agent(to_agent_id).await?;
        
        // Use agent node IDs as mailbox IDs
        let from_mailbox_id = string_to_node_id(&from_agent.id);
        let to_mailbox_id = string_to_node_id(&to_agent.id);
        
        // Create mail
        let mail = Mail::new(from_mailbox_id, to_mailbox_id, subject, body);
        let node = mail.to_node();
        
        // Create mail node
        self.storage.create_node(&node).await?;
        
        // Create edges for sender and receiver
        let from_edge = Edge::new(
            "sent_from",
            from_mailbox_id,
            mail.id,
            Properties::new(),
        );
        self.storage.create_edge(&from_edge).await?;
        
        let to_edge = Edge::new(
            "sent_to",
            mail.id,
            to_mailbox_id,
            Properties::new(),
        );
        self.storage.create_edge(&to_edge).await?;
        
        Ok(mail)
    }

    async fn get_mailbox_inbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>> {
        // Verify mailbox (agent) exists
        let _agent = self.storage.get_node(mailbox_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailboxNotFound(mailbox_id),
                _ => MailError::Storage(e),
            })?;
        
        // Get all mail where there's an edge from mail -> mailbox (sent_to)
        let incoming_edges = self.storage
            .get_edges_to(mailbox_id, Some("sent_to"))
            .await?;
        
        let mut mails = Vec::new();
        for edge in incoming_edges {
            if let Ok(mail) = self.get_mail(edge.from_node_id).await {
                mails.push(mail);
            }
        }
        
        // Sort by creation date, newest first
        mails.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(mails)
    }

    async fn get_mailbox_outbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>> {
        // Verify mailbox (agent) exists
        let _agent = self.storage.get_node(mailbox_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailboxNotFound(mailbox_id),
                _ => MailError::Storage(e),
            })?;
        
        // Get all mail where there's an edge from mailbox -> mail (sent_from)
        let outgoing_edges = self.storage
            .get_edges_from(mailbox_id, Some("sent_from"))
            .await?;
        
        let mut mails = Vec::new();
        for edge in outgoing_edges {
            if let Ok(mail) = self.get_mail(edge.to_node_id).await {
                mails.push(mail);
            }
        }
        
        // Sort by creation date, newest first
        mails.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(mails)
    }

    async fn get_recent_mail(&self, mailbox_id: MailboxId, hours: i64, limit: usize) -> Result<Vec<Mail>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(hours);
        
        // Get all mail in the inbox
        let inbox = self.get_mailbox_inbox(mailbox_id).await?;
        
        // Filter to recent mail only
        let recent: Vec<Mail> = inbox.into_iter()
            .filter(|mail| mail.created_at >= since)
            .take(limit)
            .collect();
        
        Ok(recent)
    }

    async fn mark_mail_as_read(&self, mail_id: uuid::Uuid) -> Result<Mail> {
        let mut mail = self.get_mail(mail_id).await?;
        mail.mark_as_read();
        
        let node = mail.to_node();
        self.storage.update_node(&node).await?;
        
        Ok(mail)
    }

    async fn mark_mail_as_read_by_short_id(&self, short_id: &str) -> Result<Mail> {
        // Query all mail nodes in the system
        let query = GraphQuery::new().with_node_type("mail");
        let nodes = self.storage.query_nodes(&query).await?;
        
        // Convert nodes to Mail and find matching short ID
        let short_id_lower = short_id.to_lowercase();
        let matching: Vec<_> = nodes.iter()
            .filter_map(Mail::from_node)
            .filter(|m| m.id.to_string().to_lowercase().starts_with(&short_id_lower))
            .collect();
        
        match matching.len() {
            0 => Err(MailError::MailNotFound(uuid::Uuid::nil())),
            1 => {
                let mail_id = matching[0].id;
                self.mark_mail_as_read(mail_id).await
            }
            _ => Err(MailError::InvalidOperation(
                format!("Multiple mails match short ID '{}', please use full ID", short_id)
            )),
        }
    }

    async fn check_unread_mail(&self, agent_id: AgentId) -> Result<(bool, Vec<Mail>)> {
        // Get the agent's mailbox ID
        let mailbox = self.get_agent_mailbox(agent_id).await?;
        
        // Get inbox and filter for unread
        let inbox = self.get_mailbox_inbox(mailbox.id).await?;
        let unread: Vec<Mail> = inbox.into_iter()
            .filter(|mail| !mail.read)
            .collect();
        
        let has_unread = !unread.is_empty();
        
        Ok((has_unread, unread))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::memory::InMemoryStorage;

    #[tokio::test]
    async fn test_create_agent() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("test_agent").await.unwrap();
        assert_eq!(agent.name, "test_agent");
    }

    #[tokio::test]
    async fn test_create_agent_uppercase_fails() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let result = service.create_agent("TestAgent").await;
        assert!(matches!(result, Err(MailError::InvalidAgentName(_))));
    }

    #[tokio::test]
    async fn test_create_agent_spaces_fails() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let result = service.create_agent("test agent").await;
        assert!(matches!(result, Err(MailError::InvalidAgentName(_))));
    }

    #[tokio::test]
    async fn test_create_agent_auto_creates_mailbox() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("agent").await.unwrap();
        
        // Should be able to get the mailbox for the agent
        let mailbox = service.get_agent_mailbox(agent.id.clone()).await.unwrap();
        assert_eq!(mailbox.owner_id, agent.id);
        
        // The mailbox ID should be the same as the agent's node ID
        let expected_mailbox_id = string_to_node_id(&agent.id);
        assert_eq!(mailbox.id, expected_mailbox_id);
    }

    #[tokio::test]
    async fn test_send_and_receive_mail() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        // Create two agents - mailboxes are auto-created
        let agent1 = service.create_agent("sender").await.unwrap();
        let agent2 = service.create_agent("receiver").await.unwrap();
        
        // Send mail directly between agents
        let mail = service
            .send_agent_to_agent(
                agent1.id.clone(),
                agent2.id.clone(),
                "Hello",
                "This is a test message",
            )
            .await
            .unwrap();
        
        assert_eq!(mail.subject, "Hello");
        assert_eq!(mail.body, "This is a test message");
        assert!(!mail.read);
        
        // Check receiver's inbox
        let inbox = service.get_mailbox_inbox(string_to_node_id(&agent2.id)).await.unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].subject, "Hello");
        
        // Check sender's outbox
        let outbox = service.get_mailbox_outbox(string_to_node_id(&agent1.id)).await.unwrap();
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox[0].subject, "Hello");
    }

    #[tokio::test]
    async fn test_mark_mail_as_read() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("sender").await.unwrap();
        let agent2 = service.create_agent("receiver").await.unwrap();
        
        let mail = service
            .send_agent_to_agent(agent1.id, agent2.id, "Test", "Body")
            .await
            .unwrap();
        
        assert!(!mail.read);
        
        let updated = service.mark_mail_as_read(mail.id).await.unwrap();
        assert!(updated.read);
    }

    #[tokio::test]
    async fn test_check_unread_mail() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("sender").await.unwrap();
        let agent2 = service.create_agent("receiver").await.unwrap();
        
        // Initially no unread mail
        let (has_unread, unread) = service.check_unread_mail(agent2.id.clone()).await.unwrap();
        assert!(!has_unread);
        assert!(unread.is_empty());
        
        // Send mail
        service.send_agent_to_agent(agent1.id, agent2.id.clone(), "Test", "Body").await.unwrap();
        
        // Now there is unread mail
        let (has_unread, unread) = service.check_unread_mail(agent2.id.clone()).await.unwrap();
        assert!(has_unread);
        assert_eq!(unread.len(), 1);
        
        // Mark as read
        let mail_id = unread[0].id;
        service.mark_mail_as_read(mail_id).await.unwrap();
        
        // No more unread
        let (has_unread, unread) = service.check_unread_mail(agent2.id).await.unwrap();
        assert!(!has_unread);
        assert!(unread.is_empty());
    }

    #[tokio::test]
    async fn test_get_recent_mail() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("sender").await.unwrap();
        let agent2 = service.create_agent("receiver").await.unwrap();
        
        // Send some mail
        service.send_agent_to_agent(agent1.id.clone(), agent2.id.clone(), "Recent", "Body").await.unwrap();
        
        // Get recent mail (last 24 hours)
        let recent = service.get_recent_mail(
            string_to_node_id(&agent2.id),
            24,
            10
        ).await.unwrap();
        
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].subject, "Recent");
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let fake_id = "nonexistent_agent".to_string();
        let result = service.get_agent(fake_id).await;
        
        assert!(matches!(result, Err(MailError::AgentNotFound(_))));
    }

    #[tokio::test]
    async fn test_get_agent_by_mailbox() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("test_agent").await.unwrap();
        let mailbox = service.get_agent_mailbox(agent.id.clone()).await.unwrap();
        
        // Get agent by mailbox
        let found_agent = service.get_agent_by_mailbox(mailbox.id).await.unwrap();
        assert_eq!(found_agent.id, agent.id);
        assert_eq!(found_agent.name, agent.name);
    }

    #[tokio::test]
    async fn test_list_agents() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        // Create multiple agents
        service.create_agent("agent_1").await.unwrap();
        service.create_agent("agent_2").await.unwrap();
        service.create_agent("agent_3").await.unwrap();
        
        let agents = service.list_agents().await.unwrap();
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_set_agent_status() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("test_agent").await.unwrap();
        assert_eq!(agent.status, "offline");
        
        let updated = service.set_agent_status(agent.id.clone(), "online").await.unwrap();
        assert_eq!(updated.status, "online");
        
        // Verify persisted
        let retrieved = service.get_agent(agent.id).await.unwrap();
        assert_eq!(retrieved.status, "online");
    }

    #[tokio::test]
    async fn test_multiple_mails_sorting() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("sender").await.unwrap();
        let agent2 = service.create_agent("receiver").await.unwrap();
        
        // Send multiple mails
        service.send_agent_to_agent(agent1.id.clone(), agent2.id.clone(), "First", "Body1").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        service.send_agent_to_agent(agent1.id.clone(), agent2.id.clone(), "Second", "Body2").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        service.send_agent_to_agent(agent1.id.clone(), agent2.id.clone(), "Third", "Body3").await.unwrap();
        
        let inbox = service.get_mailbox_inbox(string_to_node_id(&agent2.id)).await.unwrap();
        assert_eq!(inbox.len(), 3);
        // Should be sorted newest first
        assert_eq!(inbox[0].subject, "Third");
        assert_eq!(inbox[1].subject, "Second");
        assert_eq!(inbox[2].subject, "First");
    }
}
