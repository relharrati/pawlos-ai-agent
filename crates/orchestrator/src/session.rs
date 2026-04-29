use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use pawlos_core::types::Session;
use pawlos_core::db::Database;
use uuid::Uuid;

/// Manages all active sessions in memory; persists to SQLite on changes
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    db: Arc<Mutex<Database>>,
}

impl SessionManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            db,
        }
    }

    pub async fn create_session(&self, agent_name: &str) -> Result<Session> {
        let session = Session::new(agent_name);
        let id = session.id;

        // Persist
        {
            let db = self.db.lock().await;
            db.conn.execute(
                "INSERT INTO sessions (id, agent_name, created_at, updated_at, metadata)
                 VALUES (?1, ?2, ?3, ?4, '{}')",
                rusqlite::params![
                    id.to_string(),
                    agent_name,
                    session.created_at.to_rfc3339(),
                    session.updated_at.to_rfc3339(),
                ],
            )?;
        }

        let mut sessions = self.sessions.lock().await;
        sessions.insert(id, session.clone());
        Ok(session)
    }

    pub async fn get_or_create(&self, agent_name: &str) -> Result<Uuid> {
        let sessions = self.sessions.lock().await;
        // Return first matching session if one exists
        for (id, s) in sessions.iter() {
            if s.agent_name == agent_name {
                return Ok(*id);
            }
        }
        drop(sessions);
        let s = self.create_session(agent_name).await?;
        Ok(s.id)
    }

    pub async fn get(&self, id: Uuid) -> Option<Session> {
        self.sessions.lock().await.get(&id).cloned()
    }

    pub async fn push_message(&self, session_id: Uuid, msg: pawlos_core::types::Message) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            // Persist message
            {
                let db = self.db.lock().await;
                db.conn.execute(
                    "INSERT INTO messages (id, session_id, role, content, tool_calls, tool_call_id, timestamp)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        msg.id.to_string(),
                        session_id.to_string(),
                        format!("{:?}", msg.role).to_lowercase(),
                        msg.content,
                        msg.tool_calls.as_ref().map(|tc| serde_json::to_string(tc).ok()).flatten(),
                        msg.tool_call_id,
                        msg.timestamp.to_rfc3339(),
                    ],
                )?;
            }
            session.push(msg);
        }
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<Uuid> {
        self.sessions.lock().await.keys().cloned().collect()
    }
}
