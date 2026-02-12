use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Schedule {
    pub id: Uuid,
    pub agent_id: String,
    pub cron_expression: String,
    pub action: String,
    pub is_active: bool,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Schedule {
    pub fn new(agent_id: String, cron_expression: String, action: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            cron_expression,
            action,
            is_active: true,
            last_fired_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleId(pub Uuid);
