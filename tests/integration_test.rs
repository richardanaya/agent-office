use agent_office::services::mail::{MailService, MailServiceImpl};
use agent_office::storage::memory::InMemoryStorage;

#[tokio::test]
async fn test_full_mail_workflow() {
    // Setup
    let storage = InMemoryStorage::new();
    let service = MailServiceImpl::new(storage);
    
    // Create two agents (each auto-creates a single mailbox)
    let alice = service.create_agent("alice").await.unwrap();
    let bob = service.create_agent("bob").await.unwrap();
    let alice_id = alice.id.clone();
    let bob_id = bob.id.clone();
    println!("Created agents: Alice ({}), Bob ({})", alice_id, bob_id);
    
    // Get their mailboxes (each agent has exactly one)
    let alice_mailbox = service.get_agent_mailbox(alice.id.clone()).await.unwrap();
    let bob_mailbox = service.get_agent_mailbox(bob.id.clone()).await.unwrap();
    
    println!("Agent mailboxes (single mailbox per agent):");
    println!("  Alice mailbox: {} (same as agent ID)", alice_mailbox.id);
    println!("  Bob mailbox: {} (same as agent ID)", bob_mailbox.id);
    
    // Send mail from Alice to Bob using agent IDs directly
    let mail1 = service.send_agent_to_agent(
        alice_id.clone(),
        bob_id.clone(),
        "Hello Bob!",
        "This is Alice. Nice to meet you!",
    ).await.unwrap();
    
    println!("\nAlice sent mail to Bob:");
    println!("  Subject: {}", mail1.subject);
    println!("  Body: {}", mail1.body);
    println!("  Mail ID: {}", mail1.id);
    
    // Send reply from Bob to Alice
    let _mail2 = service.send_agent_to_agent(
        bob_id.clone(),
        alice_id.clone(),
        "Re: Hello Bob!",
        "Hi Alice! Thanks for reaching out.",
    ).await.unwrap();
    
    // Check Bob's inbox
    let bob_inbox = service.get_mailbox_inbox(bob_mailbox.id).await.unwrap();
    println!("\nBob's inbox has {} mail(s)", bob_inbox.len());
    for mail in &bob_inbox {
        let sender = service.get_agent_by_mailbox(mail.from_mailbox_id).await.unwrap();
        let status = if mail.read { "Read" } else { "Unread" };
        println!("  [{}] {} - {} (from {})", status, mail.id, mail.subject, sender.name);
    }
    
    // Check Alice's inbox
    let alice_inbox = service.get_mailbox_inbox(alice_mailbox.id).await.unwrap();
    println!("\nAlice's inbox has {} mail(s)", alice_inbox.len());
    for mail in &alice_inbox {
        let sender = service.get_agent_by_mailbox(mail.from_mailbox_id).await.unwrap();
        let status = if mail.read { "Read" } else { "Unread" };
        println!("  [{}] {} - {} (from {})", status, mail.id, mail.subject, sender.name);
    }
    
    // Mark mail as read
    service.mark_mail_as_read(mail1.id).await.unwrap();
    println!("\nMarked mail {} as read", mail1.id);
    
    // Check Alice's outbox
    let alice_outbox = service.get_mailbox_outbox(alice_mailbox.id).await.unwrap();
    println!("\nAlice's outbox has {} mail(s)", alice_outbox.len());
    for mail in &alice_outbox {
        let recipient = service.get_agent_by_mailbox(mail.to_mailbox_id).await.unwrap();
        println!("  {} - {} (to {})", mail.id, mail.subject, recipient.name);
    }
    
    // List all agents
    let agents = service.list_agents().await.unwrap();
    println!("\nAll agents: {}", agents.len());
    for agent in agents {
        println!("  {} - {}", agent.id, agent.name);
    }
    
    // Each agent has exactly one mailbox (their own agent ID)
    println!("\nEach agent has exactly one mailbox (agent ID = mailbox ID)");
}
