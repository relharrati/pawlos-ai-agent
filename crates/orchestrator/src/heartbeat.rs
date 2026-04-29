use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use core::Config;
use crate::turn::run_turn;
use provider::registry::ProviderRegistry;
use crate::session::SessionManager;
use core::db::Database;

/// Periodic heartbeat scheduler
pub struct Heartbeat {
    interval_secs: u64,
    enabled: bool,
    agent_name: String,
}

impl Heartbeat {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            interval_secs: cfg.heartbeat.interval_secs,
            enabled: cfg.heartbeat.enabled,
            agent_name: cfg.agent.name.clone(),
        }
    }

    /// Spawn a background task that fires heartbeat ticks
    pub fn start(
        self,
        db: Arc<Mutex<Database>>,
        session_mgr: Arc<SessionManager>,
        provider: Arc<ProviderRegistry>,
        model: String,
    ) {
        if !self.enabled {
            tracing::info!("Heartbeat disabled.");
            return;
        }

        let interval = std::time::Duration::from_secs(self.interval_secs);
        let agent_name = self.agent_name.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                tracing::info!("💓 Heartbeat tick");

                if let Err(e) = Self::tick(&agent_name, &session_mgr, &provider, &model).await {
                    tracing::warn!("Heartbeat tick error: {e}");
                }
            }
        });
    }

    async fn tick(
        agent_name: &str,
        session_mgr: &Arc<SessionManager>,
        provider: &Arc<ProviderRegistry>,
        model: &str,
    ) -> Result<()> {
        // Get or create a background heartbeat session
        let session_id = session_mgr.get_or_create(&format!("{agent_name}_heartbeat")).await?;

        let heartbeat_prompt = "You are performing a scheduled heartbeat self-check. \
            Briefly summarise any pending tasks, check if memory consolidation is needed, \
            and note any ideas or reminders. Keep it under 3 sentences.";

        let (response, _) = run_turn(
            session_id,
            heartbeat_prompt.to_string(),
            agent_name,
            session_mgr,
            provider,
            model,
            false, // no tool calls during heartbeat
        )
        .await?;

        tracing::info!("Heartbeat response: {response}");
        Ok(())
    }
}
