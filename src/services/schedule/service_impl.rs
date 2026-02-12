use async_trait::async_trait;
use chrono::{DateTime, Duration, Timelike, Utc};
use cron::Schedule as CronSchedule;
use sqlx::{Pool, Postgres, Row};
use std::str::FromStr;
use uuid::Uuid;

use crate::services::schedule::{Result, ScheduleError, ScheduleService};
use crate::services::schedule::domain::Schedule;

pub struct ScheduleServiceImpl {
    pool: Pool<Postgres>,
}

impl ScheduleServiceImpl {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Helper to validate cron expression
    /// Supports both 5-field (standard) and 6-field (with seconds) formats
    fn validate_cron(&self, expression: &str) -> Result<CronSchedule> {
        // First try the expression as-is
        if let Ok(schedule) = CronSchedule::from_str(expression) {
            return Ok(schedule);
        }
        
        // If that fails, try prepending "0 " for seconds (convert 5-field to 6-field)
        let with_seconds = format!("0 {}", expression);
        CronSchedule::from_str(&with_seconds)
            .map_err(|e| ScheduleError::InvalidCronExpression(format!("{}: {}", expression, e)))
    }

    /// Helper to check if schedule should fire
    /// Check if schedule should fire for current minute
    fn should_fire(&self, schedule: &Schedule, current_time: DateTime<Utc>) -> bool {
        eprintln!("DEBUG should_fire: checking schedule id={}, cron='{}', is_active={}", 
            schedule.id, schedule.cron_expression, schedule.is_active);
        
        if !schedule.is_active {
            eprintln!("DEBUG should_fire: inactive, returning false");
            return false;
        }

        let cron = match self.validate_cron(&schedule.cron_expression) {
            Ok(c) => {
                eprintln!("DEBUG should_fire: cron validated successfully");
                c
            }
            Err(e) => {
                eprintln!("DEBUG should_fire: cron validation failed: {}", e);
                return false;
            }
        };

        // Get the current minute boundaries
        let current_minute_start = current_time
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let current_minute_end = current_minute_start + Duration::minutes(1);
        
        eprintln!("DEBUG should_fire: current_time={}, minute_start={}, minute_end={}",
            current_time.format("%Y-%m-%d %H:%M:%S"),
            current_minute_start.format("%Y-%m-%d %H:%M:%S"),
            current_minute_end.format("%Y-%m-%d %H:%M:%S"));

        // Check if last_fired_at was within current minute
        if let Some(last_fired) = schedule.last_fired_at {
            eprintln!("DEBUG should_fire: last_fired={}", last_fired.format("%Y-%m-%d %H:%M:%S"));
            if last_fired >= current_minute_start && last_fired < current_minute_end {
                eprintln!("DEBUG should_fire: already fired this minute, returning false");
                return false; // Already fired this minute
            }
        } else {
            eprintln!("DEBUG should_fire: never fired before");
        }

        // Check if cron would fire during this minute
        let upcoming: Vec<_> = cron
            .after(&current_minute_start)
            .take(1)
            .collect();
        
        eprintln!("DEBUG should_fire: upcoming times found: {}", upcoming.len());

        if let Some(next_time) = upcoming.first() {
            eprintln!("DEBUG should_fire: next cron occurrence at {} (minute_start={}, minute_end={})",
                next_time.format("%Y-%m-%d %H:%M:%S"),
                current_minute_start.format("%Y-%m-%d %H:%M:%S"),
                current_minute_end.format("%Y-%m-%d %H:%M:%S"));
            let should = *next_time >= current_minute_start && *next_time < current_minute_end;
            eprintln!("DEBUG should_fire: next_time in current minute? {} -> returning {}", 
                should, should);
            should
        } else {
            eprintln!("DEBUG should_fire: no upcoming times, returning false");
            false
        }
    }
}

#[async_trait]
impl ScheduleService for ScheduleServiceImpl {
    async fn create_schedule(
        &self,
        agent_id: String,
        cron_expression: String,
        action: String,
    ) -> Result<Schedule> {
        // Validate cron expression
        self.validate_cron(&cron_expression)?;

        let schedule = Schedule::new(agent_id, cron_expression, action);

        sqlx::query(
            r#"
            INSERT INTO schedules (id, agent_id, cron_expression, action, is_active, last_fired_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(schedule.id)
        .bind(&schedule.agent_id)
        .bind(&schedule.cron_expression)
        .bind(&schedule.action)
        .bind(schedule.is_active)
        .bind(schedule.last_fired_at)
        .bind(schedule.created_at)
        .bind(schedule.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ScheduleError::Storage(e.to_string()))?;

        Ok(schedule)
    }

    async fn get_schedule(&self, id: Uuid) -> Result<Schedule> {
        let row = sqlx::query(
            r#"
            SELECT id, agent_id, cron_expression, action, is_active, last_fired_at, created_at, updated_at
            FROM schedules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ScheduleError::ScheduleNotFound(id),
            _ => ScheduleError::Storage(e.to_string()),
        })?;

        Ok(Schedule {
            id: row.get("id"),
            agent_id: row.get("agent_id"),
            cron_expression: row.get("cron_expression"),
            action: row.get("action"),
            is_active: row.get("is_active"),
            last_fired_at: row.get("last_fired_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_schedules_by_agent(&self, agent_id: &str) -> Result<Vec<Schedule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, cron_expression, action, is_active, last_fired_at, created_at, updated_at
            FROM schedules
            WHERE agent_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ScheduleError::Storage(e.to_string()))?;

        let schedules = rows
            .iter()
            .map(|row| Schedule {
                id: row.get("id"),
                agent_id: row.get("agent_id"),
                cron_expression: row.get("cron_expression"),
                action: row.get("action"),
                is_active: row.get("is_active"),
                last_fired_at: row.get("last_fired_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(schedules)
    }

    async fn update_schedule(
        &self,
        id: Uuid,
        cron_expression: Option<String>,
        action: Option<String>,
    ) -> Result<Schedule> {
        // Validate cron if provided
        if let Some(ref cron) = cron_expression {
            self.validate_cron(cron)?;
        }

        let schedule = self.get_schedule(id).await?;

        let new_cron = cron_expression.unwrap_or_else(|| schedule.cron_expression.clone());
        let new_action = action.unwrap_or_else(|| schedule.action.clone());

        sqlx::query(
            r#"
            UPDATE schedules
            SET cron_expression = $1, action = $2, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(&new_cron)
        .bind(&new_action)
        .bind(Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| ScheduleError::Storage(e.to_string()))?;

        let mut updated = schedule;
        updated.cron_expression = new_cron;
        updated.action = new_action;
        updated.updated_at = Utc::now();

        Ok(updated)
    }

    async fn delete_schedule(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM schedules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ScheduleError::Storage(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ScheduleError::ScheduleNotFound(id));
        }

        Ok(())
    }

    async fn toggle_schedule(&self, id: Uuid) -> Result<Schedule> {
        let schedule = self.get_schedule(id).await?;
        let new_status = !schedule.is_active;

        sqlx::query(
            r#"
            UPDATE schedules
            SET is_active = $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(new_status)
        .bind(Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| ScheduleError::Storage(e.to_string()))?;

        let mut updated = schedule;
        updated.is_active = new_status;
        updated.updated_at = Utc::now();

        Ok(updated)
    }

    async fn check_and_fire_schedules(
        &self,
        agent_id: &str,
        current_time: DateTime<Utc>,
    ) -> Result<Vec<String>> {
        let schedules = self.list_schedules_by_agent(agent_id).await?;
        eprintln!("DEBUG: check_and_fire_schedules found {} schedules for agent {}", schedules.len(), agent_id);
        let mut fired_actions = Vec::new();

        for (i, schedule) in schedules.iter().enumerate() {
            eprintln!("DEBUG: Checking schedule {}: id={}, cron='{}', active={}", 
                i, schedule.id, schedule.cron_expression, schedule.is_active);
            let should_fire = self.should_fire(schedule, current_time);
            eprintln!("DEBUG: should_fire returned: {}", should_fire);
            if should_fire {
                eprintln!("DEBUG: Schedule {} will fire! Updating last_fired_at...", schedule.id);
                // Update last_fired_at
                sqlx::query(
                    r#"
                    UPDATE schedules
                    SET last_fired_at = $1, updated_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(current_time)
                .bind(Utc::now())
                .bind(schedule.id)
                .execute(&self.pool)
                .await
                .map_err(|e| ScheduleError::Storage(e.to_string()))?;

                fired_actions.push(schedule.action.clone());
                eprintln!("DEBUG: Schedule fired successfully, action: '{}'", schedule.action);
            }
        }

        eprintln!("DEBUG: check_and_fire_schedules returning {} fired actions", fired_actions.len());
        Ok(fired_actions)
    }

    fn get_next_run(&self, schedule: &Schedule, current_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        if !schedule.is_active {
            return None;
        }

        let cron = self.validate_cron(&schedule.cron_expression).ok()?;
        cron.after(&current_time).next()
    }
}
