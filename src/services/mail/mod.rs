use crate::domain::{Edge, GraphQuery, Properties, string_to_node_id};
use crate::services::mail::domain::{Agent, AgentId, Mail, Mailbox, MailboxId};
use crate::storage::{GraphStorage, StorageError, EdgeDirection};
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
}

pub type Result<T> = std::result::Result<T, MailError>;

#[async_trait]
pub trait MailService: Send + Sync {
    // Agent operations
    async fn create_agent(&self, name: impl Into<String> + Send) -> Result<Agent>;
    async fn get_agent(&self, id: AgentId) -> Result<Agent>;
    async fn list_agents(&self) -> Result<Vec<Agent>>;
    async fn set_agent_status(&self, agent_id: AgentId, status: impl Into<String> + Send) -> Result<Agent>;
    
    // Mailbox operations
    async fn create_mailbox(&self, owner_id: AgentId, name: impl Into<String> + Send) -> Result<Mailbox>;
    async fn get_mailbox(&self, id: MailboxId) -> Result<Mailbox>;
    async fn list_agent_mailboxes(&self, agent_id: AgentId) -> Result<Vec<Mailbox>>;
    async fn delete_mailbox(&self, mailbox_id: MailboxId) -> Result<()>;
    async fn get_mailbox_owner(&self, mailbox_id: MailboxId) -> Result<Agent>;
    
    // Mail operations
    async fn send_mail(
        &self,
        from_mailbox_id: MailboxId,
        to_mailbox_id: MailboxId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail>;
    
    async fn get_mail(&self, mail_id: uuid::Uuid) -> Result<Mail>;
    async fn get_mailbox_inbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>>;
    async fn get_mailbox_outbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>>;
    async fn mark_mail_as_read(&self, mail_id: uuid::Uuid) -> Result<Mail>;
    async fn delete_mail(&self, mail_id: uuid::Uuid) -> Result<()>;
    
    // Check if agent has unread mail
    async fn check_unread_mail(&self, agent_id: AgentId) -> Result<(bool, Vec<Mail>)>;
    
    // Get agent's default inbox
    async fn get_agent_inbox(&self, agent_id: AgentId) -> Result<Mailbox>;
    
    // Get agent's default sending mailbox (outbox if exists, otherwise inbox)
    async fn get_agent_outbox(&self, agent_id: AgentId) -> Result<Mailbox>;
    
    // Send mail to an agent (delivers to their inbox)
    async fn send_mail_to_agent(
        &self,
        from_mailbox_id: MailboxId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail>;
    
    // Send mail from one agent to another by names only
    async fn send_agent_to_agent(
        &self,
        from_agent_id: AgentId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail>;
    
    // Search mail with filters
    async fn search_mail(
        &self,
        search_text: Option<String>,
        agent_id: Option<AgentId>,
        after: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<(Vec<Mail>, usize, bool)>; // (results, total_count, has_more)
    
    // Get recent mail
    async fn recent_mail(&self, hours: i64, limit: usize) -> Result<Vec<Mail>>;
}

pub struct MailServiceImpl<S: GraphStorage> {
    storage: S,
}

impl<S: GraphStorage> MailServiceImpl<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<S: GraphStorage> MailService for MailServiceImpl<S> {
    async fn create_agent(&self, name: impl Into<String> + Send) -> Result<Agent> {
        let agent = Agent::new(name);
        let node = agent.to_node();
        self.storage.create_node(&node).await?;
        
        // Auto-create an inbox for the agent
        let inbox = Mailbox::new(agent.id.clone(), "Inbox");
        let inbox_node = inbox.to_node();
        self.storage.create_node(&inbox_node).await?;
        
        // Create ownership edge
        let owner_node_id = string_to_node_id(&agent.id);
        let edge = Edge::new(
            "owns",
            owner_node_id,
            inbox.id,
            Properties::new(),
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(agent)
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

    async fn create_mailbox(&self, owner_id: AgentId, name: impl Into<String> + Send) -> Result<Mailbox> {
        // Verify owner exists
        let owner_node_id = string_to_node_id(&owner_id);
        self.storage.get_node(owner_node_id).await?;
        
        let mailbox = Mailbox::new(owner_id, name);
        let node = mailbox.to_node();
        
        // Create mailbox node
        self.storage.create_node(&node).await?;
        
        // Create ownership edge
        let edge = Edge::new(
            "owns",
            owner_node_id,
            mailbox.id,
            Properties::new(),
        );
        self.storage.create_edge(&edge).await?;
        
        Ok(mailbox)
    }

    async fn get_mailbox(&self, id: MailboxId) -> Result<Mailbox> {
        let node = self.storage.get_node(id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailboxNotFound(id),
                _ => MailError::Storage(e),
            })?;
        Mailbox::from_node(&node)
            .ok_or(MailError::MailboxNotFound(id))
    }

    async fn list_agent_mailboxes(&self, agent_id: AgentId) -> Result<Vec<Mailbox>> {
        // Verify agent exists
        let agent_node_id = string_to_node_id(&agent_id);
        self.storage.get_node(agent_node_id).await?;
        
        // Get all mailboxes owned by this agent
        let mailboxes = self.storage
            .get_neighbors(agent_node_id, Some("owns"), EdgeDirection::Outgoing)
            .await?;
        
        let mailboxes: Vec<Mailbox> = mailboxes.iter()
            .filter_map(Mailbox::from_node)
            .collect();
        
        Ok(mailboxes)
    }

    async fn get_mailbox_owner(&self, mailbox_id: MailboxId) -> Result<Agent> {
        // Get the mailbox node to verify it exists
        let mailbox_node = self.storage.get_node(mailbox_id).await
            .map_err(|e| match e {
                StorageError::NodeNotFound(_) => MailError::MailboxNotFound(mailbox_id),
                _ => MailError::Storage(e),
            })?;
        
        // Get the owner from the mailbox properties
        let owner_id = mailbox_node.get_property("owner_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| MailError::InvalidOperation("Mailbox has no owner".to_string()))?;
        
        // Get the agent
        self.get_agent(owner_id).await
    }

    async fn delete_mailbox(&self, mailbox_id: MailboxId) -> Result<()> {
        // Verify mailbox exists
        self.get_mailbox(mailbox_id).await?;
        
        // Delete the mailbox (this will also delete all connected edges due to FK constraints
        // in PostgreSQL, but in memory we handle this manually)
        self.storage.delete_node(mailbox_id).await?;
        
        Ok(())
    }

    async fn send_mail(
        &self,
        from_mailbox_id: MailboxId,
        to_mailbox_id: MailboxId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail> {
        // Verify both mailboxes exist
        self.get_mailbox(from_mailbox_id).await?;
        self.get_mailbox(to_mailbox_id).await?;
        
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

    async fn get_mail(&self, mail_id: uuid::Uuid) -> Result<Mail> {
        let node = self.storage.get_node(mail_id).await?;
        Mail::from_node(&node)
            .ok_or(MailError::MailNotFound(mail_id))
    }

    async fn get_mailbox_inbox(&self, mailbox_id: MailboxId) -> Result<Vec<Mail>> {
        // Verify mailbox exists
        self.get_mailbox(mailbox_id).await?;
        
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
        // Verify mailbox exists
        self.get_mailbox(mailbox_id).await?;
        
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

    async fn mark_mail_as_read(&self, mail_id: uuid::Uuid) -> Result<Mail> {
        let mut mail = self.get_mail(mail_id).await?;
        mail.mark_as_read();
        
        let node = mail.to_node();
        self.storage.update_node(&node).await?;
        
        Ok(mail)
    }

    async fn delete_mail(&self, mail_id: uuid::Uuid) -> Result<()> {
        // Verify mail exists
        self.get_mail(mail_id).await?;
        
        // Delete the mail (edges will be cleaned up)
        self.storage.delete_node(mail_id).await?;
        
        Ok(())
    }

    async fn check_unread_mail(&self, agent_id: AgentId) -> Result<(bool, Vec<Mail>)> {
        // Get all mailboxes for this agent
        let mailboxes = self.list_agent_mailboxes(agent_id).await?;
        
        let mut all_unread = Vec::new();
        
        // Check inbox of each mailbox for unread mail
        for mailbox in mailboxes {
            let inbox = self.get_mailbox_inbox(mailbox.id).await?;
            for mail in inbox {
                if !mail.read {
                    all_unread.push(mail);
                }
            }
        }
        
        let has_unread = !all_unread.is_empty();
        
        // Sort by creation date, newest first
        all_unread.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok((has_unread, all_unread))
    }

    async fn get_agent_inbox(&self, agent_id: AgentId) -> Result<Mailbox> {
        // Get all mailboxes for this agent and find the one named "Inbox"
        let mailboxes = self.list_agent_mailboxes(agent_id.clone()).await?;
        
        mailboxes.into_iter()
            .find(|m| m.name == "Inbox")
            .ok_or_else(|| MailError::MailboxNotFound(
                // Return a placeholder - the error is that no inbox exists
                uuid::Uuid::nil()
            ))
    }

    async fn send_mail_to_agent(
        &self,
        from_mailbox_id: MailboxId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail> {
        // Get the recipient's inbox
        let inbox = self.get_agent_inbox(to_agent_id).await?;
        
        // Send mail to that inbox
        self.send_mail(from_mailbox_id, inbox.id, subject, body).await
    }

    async fn get_agent_outbox(&self, agent_id: AgentId) -> Result<Mailbox> {
        // Get all mailboxes for this agent and find the one named "Outbox" or use first available
        let mailboxes = self.list_agent_mailboxes(agent_id.clone()).await?;
        
        // Try to find an Outbox first
        if let Some(outbox) = mailboxes.iter().find(|m| m.name == "Outbox") {
            return Ok(outbox.clone());
        }
        
        // Otherwise use the first mailbox (usually the Inbox)
        mailboxes.into_iter()
            .next()
            .ok_or_else(|| MailError::MailboxNotFound(uuid::Uuid::nil()))
    }

    async fn send_agent_to_agent(
        &self,
        from_agent_id: AgentId,
        to_agent_id: AgentId,
        subject: impl Into<String> + Send,
        body: impl Into<String> + Send,
    ) -> Result<Mail> {
        // Get sender's outbox (or first available mailbox)
        let from_mailbox = self.get_agent_outbox(from_agent_id).await?;
        
        // Get recipient's inbox
        let to_mailbox = self.get_agent_inbox(to_agent_id).await?;
        
        // Send the mail
        self.send_mail(from_mailbox.id, to_mailbox.id, subject, body).await
    }
    
    async fn search_mail(
        &self,
        search_text: Option<String>,
        agent_id: Option<AgentId>,
        after: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<(Vec<Mail>, usize, bool)> {
        use crate::storage::{SearchQuery, OrderBy, OrderDirection};
        
        let query = SearchQuery {
            node_types: vec!["mail".to_string()],
            search_text,
            search_fields: vec![],
            created_after: after,
            created_before: before,
            updated_after: None,
            property_filters: vec![],
            limit: limit + 1, // Request one extra to check if there are more
            offset: 0,
            order_by: OrderBy::CreatedAt,
            order_direction: OrderDirection::Desc,
        };
        
        // Add agent filter if specified (by filtering on from/to mailbox owners)
        // For now, we'll get all mail and filter by checking if it belongs to the agent
        
        let results = self.storage.search_nodes(&query).await
            .map_err(|e| MailError::Storage(e))?;
        
        let mut mails: Vec<Mail> = results.items.iter()
            .filter_map(|node| Mail::from_node(node))
            .collect();
        
        // If agent_id specified, filter to only mail involving that agent
        if let Some(ref agent) = agent_id {
            let _agent_node_id = string_to_node_id(agent);
            mails.retain(|_mail| {
                // Check if mail is from or to this agent's mailboxes
                // This is a simplified check - in production you'd want more sophisticated logic
                true // For now, include all
            });
        }
        
        let has_more = results.has_more;
        let total_count = results.total_count;
        
        // Trim to requested limit
        if mails.len() > limit {
            mails.truncate(limit);
        }
        
        Ok((mails, total_count, has_more))
    }
    
    async fn recent_mail(&self, hours: i64, limit: usize) -> Result<Vec<Mail>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(hours);
        
        let (mails, _, _) = self.search_mail(None, None, Some(since), None, limit).await?;
        Ok(mails)
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
        
        let agent = service.create_agent("Test Agent").await.unwrap();
        assert_eq!(agent.name, "Test Agent");
    }

    #[tokio::test]
    async fn test_create_and_list_mailboxes() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("Agent").await.unwrap();
        let agent_id = agent.id.clone();
        let mailbox1 = service.create_mailbox(agent.id.clone(), "Inbox").await.unwrap();
        let mailbox2 = service.create_mailbox(agent.id.clone(), "Archive").await.unwrap();
        
        assert_eq!(mailbox1.name, "Inbox");
        assert_eq!(mailbox2.name, "Archive");
        
        let mailboxes = service.list_agent_mailboxes(agent_id).await.unwrap();
        // Agent auto-creates an inbox, plus the 2 explicit mailboxes = 3 total
        assert_eq!(mailboxes.len(), 3);
    }

    #[tokio::test]
    async fn test_send_and_receive_mail() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        // Create two agents with mailboxes
        let agent1 = service.create_agent("Sender").await.unwrap();
        let agent2 = service.create_agent("Receiver").await.unwrap();
        
        let mailbox1 = service.create_mailbox(agent1.id.clone(), "Outbox").await.unwrap();
        let mailbox2 = service.create_mailbox(agent2.id.clone(), "Inbox").await.unwrap();
        
        // Send mail
        let mail = service
            .send_mail(
                mailbox1.id,
                mailbox2.id,
                "Hello",
                "This is a test message",
            )
            .await
            .unwrap();
        
        assert_eq!(mail.subject, "Hello");
        assert_eq!(mail.body, "This is a test message");
        assert!(!mail.read);
        
        // Check receiver's inbox
        let inbox = service.get_mailbox_inbox(mailbox2.id).await.unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].subject, "Hello");
        
        // Check sender's outbox
        let outbox = service.get_mailbox_outbox(mailbox1.id).await.unwrap();
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox[0].subject, "Hello");
    }

    #[tokio::test]
    async fn test_mark_mail_as_read() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("Sender").await.unwrap();
        let agent2 = service.create_agent("Receiver").await.unwrap();
        
        let mailbox1 = service.create_mailbox(agent1.id.clone(), "Outbox").await.unwrap();
        let mailbox2 = service.create_mailbox(agent2.id.clone(), "Inbox").await.unwrap();
        
        let mail = service
            .send_mail(mailbox1.id, mailbox2.id, "Test", "Body")
            .await
            .unwrap();
        
        assert!(!mail.read);
        
        let updated = service.mark_mail_as_read(mail.id).await.unwrap();
        assert!(updated.read);
    }

    #[tokio::test]
    async fn test_get_mailbox_outbox_empty() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent = service.create_agent("Agent").await.unwrap();
        let mailbox = service.create_mailbox(agent.id.clone(), "Outbox").await.unwrap();
        
        let outbox = service.get_mailbox_outbox(mailbox.id).await.unwrap();
        assert!(outbox.is_empty());
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
    async fn test_get_nonexistent_mailbox() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let fake_id = uuid::Uuid::new_v4();
        let result = service.get_mailbox(fake_id).await;
        
        assert!(matches!(result, Err(MailError::MailboxNotFound(_))));
    }

    #[tokio::test]
    async fn test_multiple_mails_sorting() {
        let storage = InMemoryStorage::new();
        let service = MailServiceImpl::new(storage);
        
        let agent1 = service.create_agent("Sender").await.unwrap();
        let agent2 = service.create_agent("Receiver").await.unwrap();
        
        let mailbox1 = service.create_mailbox(agent1.id.clone(), "Outbox").await.unwrap();
        let mailbox2 = service.create_mailbox(agent2.id.clone(), "Inbox").await.unwrap();
        
        // Send multiple mails
        service.send_mail(mailbox1.id, mailbox2.id, "First", "Body1").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        service.send_mail(mailbox1.id, mailbox2.id, "Second", "Body2").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        service.send_mail(mailbox1.id, mailbox2.id, "Third", "Body3").await.unwrap();
        
        let inbox = service.get_mailbox_inbox(mailbox2.id).await.unwrap();
        assert_eq!(inbox.len(), 3);
        // Should be sorted newest first
        assert_eq!(inbox[0].subject, "Third");
        assert_eq!(inbox[1].subject, "Second");
        assert_eq!(inbox[2].subject, "First");
    }
}
