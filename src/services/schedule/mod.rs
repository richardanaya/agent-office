use crate::services::schedule::domain::Schedule;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

pub mod domain;
pub mod service_impl;

pub use service_impl::ScheduleServiceImpl;

#[derive(Error, Debug)]
pub enum ScheduleError {
    #[error("Schedule not found: {0}")]
    ScheduleNotFound(Uuid),

    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid schedule ID: {0}")]
    InvalidScheduleId(String),
}

pub type Result<T> = std::result::Result<T, ScheduleError>;

#[async_trait]
pub trait ScheduleService: Send + Sync {
    /// Create a new schedule
    async fn create_schedule(
        &self,
        agent_id: String,
        cron_expression: String,
        action: String,
    ) -> Result<Schedule>;

    /// Get a schedule by ID
    async fn get_schedule(&self, id: Uuid) -> Result<Schedule>;

    /// List all schedules for an agent
    async fn list_schedules_by_agent(&self, agent_id: &str) -> Result<Vec<Schedule>>;

    /// Update a schedule
    async fn update_schedule(
        &self,
        id: Uuid,
        cron_expression: Option<String>,
        action: Option<String>,
    ) -> Result<Schedule>;

    /// Delete a schedule
    async fn delete_schedule(&self, id: Uuid) -> Result<()>;

    /// Toggle schedule on/off
    async fn toggle_schedule(&self, id: Uuid) -> Result<Schedule>;

    /// Check schedules for an agent and return fired actions
    /// This checks if any active schedule should fire for the current time
    /// and updates last_fired_at to prevent duplicate firing
    async fn check_and_fire_schedules(
        &self,
        agent_id: &str,
        current_time: DateTime<Utc>,
    ) -> Result<Vec<String>>;

    /// Get next predicted run time for a schedule
    fn get_next_run(&self, schedule: &Schedule, current_time: DateTime<Utc>) -> Option<DateTime<Utc>>;
}
