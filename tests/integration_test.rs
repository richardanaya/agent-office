use agent_office::services::mail::{MailService, MailServiceImpl};
use agent_office::storage::memory::InMemoryStorage;

#[tokio::test]
async fn test_full_mail_workflow() {
    // Setup
    let storage = InMemoryStorage::new();
    let service = MailServiceImpl::new(storage);
    
    // Create two agents
    let alice = service.create_agent("Alice").await.unwrap();
    let bob = service.create_agent("Bob").await.unwrap();
    let alice_id = alice.id.clone();
    let bob_id = bob.id.clone();
    println!("Created agents: Alice ({}), Bob ({})", alice_id, bob_id);
    
    // Create mailboxes
    let alice_inbox = service.create_mailbox(alice.id.clone(), "Inbox").await.unwrap();
    let alice_outbox = service.create_mailbox(alice.id.clone(), "Outbox").await.unwrap();
    let bob_inbox = service.create_mailbox(bob.id.clone(), "Inbox").await.unwrap();
    
    println!("Created mailboxes:");
    println!("  Alice Inbox: {}", alice_inbox.id);
    println!("  Alice Outbox: {}", alice_outbox.id);
    println!("  Bob Inbox: {}", bob_inbox.id);
    
    // Send mail from Alice to Bob
    let mail1 = service.send_mail(
        alice_outbox.id,
        bob_inbox.id,
        "Hello Bob!",
        "This is Alice. Nice to meet you!",
    ).await.unwrap();
    
    println!("\nAlice sent mail to Bob:");
    println!("  Subject: {}", mail1.subject);
    println!("  Body: {}", mail1.body);
    println!("  Mail ID: {}", mail1.id);
    
    // Send reply from Bob to Alice
    let _mail2 = service.send_mail(
        bob_inbox.id,  // Bob sends from his inbox (simplified model)
        alice_inbox.id,
        "Re: Hello Bob!",
        "Hi Alice! Thanks for reaching out.",
    ).await.unwrap();
    
    // Check Bob's inbox
    let bob_inbox_contents = service.get_mailbox_inbox(bob_inbox.id).await.unwrap();
    println!("\nBob's inbox has {} mail(s)", bob_inbox_contents.len());
    for mail in &bob_inbox_contents {
        let status = if mail.read { "Read" } else { "Unread" };
        println!("  [{}] {} - {} (from {})", status, mail.id, mail.subject, mail.from_mailbox_id);
    }
    
    // Check Alice's inbox
    let alice_inbox_contents = service.get_mailbox_inbox(alice_inbox.id).await.unwrap();
    println!("\nAlice's inbox has {} mail(s)", alice_inbox_contents.len());
    for mail in &alice_inbox_contents {
        let status = if mail.read { "Read" } else { "Unread" };
        println!("  [{}] {} - {} (from {})", status, mail.id, mail.subject, mail.from_mailbox_id);
    }
    
    // Mark mail as read
    service.mark_mail_as_read(mail1.id).await.unwrap();
    println!("\nMarked mail {} as read", mail1.id);
    
    // Check Alice's outbox
    let alice_outbox_contents = service.get_mailbox_outbox(alice_outbox.id).await.unwrap();
    println!("\nAlice's outbox has {} mail(s)", alice_outbox_contents.len());
    for mail in &alice_outbox_contents {
        println!("  {} - {} (to {})", mail.id, mail.subject, mail.to_mailbox_id);
    }
    
    // List all agents
    let agents = service.list_agents().await.unwrap();
    println!("\nAll agents: {}", agents.len());
    for agent in agents {
        println!("  {} - {}", agent.id, agent.name);
    }
    
    // List Alice's mailboxes
    let alice_mailboxes = service.list_agent_mailboxes(alice_id).await.unwrap();
    println!("\nAlice's mailboxes: {}", alice_mailboxes.len());
    for mailbox in alice_mailboxes {
        println!("  {} - {}", mailbox.id, mailbox.name);
    }
}
