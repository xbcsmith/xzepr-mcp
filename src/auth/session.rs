//! Session management for StreamableHTTP transport
//!
//! This module provides secure session management with:
//! - Session binding to JWT sub claim
//! - Session timeouts (absolute and idle)
//! - Session rotation
//! - Secure session ID generation

use crate::error::{AuthError, Result};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use ulid::Ulid;

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID (ULID)
    pub id: String,

    /// User ID (JWT sub claim)
    pub user_id: String,

    /// User email
    pub email: Option<String>,

    /// User display name
    pub display_name: String,

    /// JWT scopes
    pub scopes: Vec<String>,

    /// Session creation time
    pub created_at: SystemTime,

    /// Last activity time
    pub last_activity: SystemTime,

    /// Session expiration time
    pub expires_at: SystemTime,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Check if session is idle (no activity within timeout)
    pub fn is_idle(&self, idle_timeout: Duration) -> bool {
        if let Ok(elapsed) = SystemTime::now().duration_since(self.last_activity) {
            elapsed > idle_timeout
        } else {
            true
        }
    }

    /// Update last activity time
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// Check if session has required scope
    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scopes.iter().any(|s| s == required_scope)
    }
}

/// Session manager
#[derive(Clone)]
pub struct SessionManager {
    /// Session cache (session_id -> Session)
    sessions: Arc<Cache<String, Session>>,

    /// Session timeout duration
    session_timeout: Duration,

    /// Session idle timeout duration
    idle_timeout: Duration,

    /// Enable session binding to JWT sub
    enable_binding: bool,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// # Arguments
    ///
    /// * `session_timeout` - Absolute session timeout
    /// * `idle_timeout` - Idle timeout (no activity)
    /// * `enable_binding` - Enable session binding to JWT sub claim
    ///
    /// # Returns
    ///
    /// Returns a new `SessionManager` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::auth::SessionManager;
    /// use std::time::Duration;
    ///
    /// let manager = SessionManager::new(
    ///     Duration::from_secs(3600),  // 1 hour
    ///     Duration::from_secs(1800),  // 30 minutes
    ///     true,
    /// );
    /// ```
    pub fn new(session_timeout: Duration, idle_timeout: Duration, enable_binding: bool) -> Self {
        let sessions = Cache::builder()
            .time_to_live(session_timeout)
            .max_capacity(10_000)
            .build();

        Self {
            sessions: Arc::new(sessions),
            session_timeout,
            idle_timeout,
            enable_binding,
        }
    }

    /// Create a new session
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID from JWT sub claim
    /// * `email` - User email
    /// * `display_name` - User display name
    /// * `scopes` - JWT scopes
    ///
    /// # Returns
    ///
    /// Returns the session ID
    ///
    /// # Examples
    ///
    /// ```
    /// use xzepr_mcp::auth::SessionManager;
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = SessionManager::new(
    ///     Duration::from_secs(3600),
    ///     Duration::from_secs(1800),
    ///     true,
    /// );
    ///
    /// let session_id = manager.create_session(
    ///     "user123".to_string(),
    ///     Some("user@example.com".to_string()),
    ///     "Test User".to_string(),
    ///     vec!["xzepr:read".to_string(), "xzepr:write".to_string()],
    /// ).await;
    ///
    /// println!("Created session: {}", session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_session(
        &self,
        user_id: String,
        email: Option<String>,
        display_name: String,
        scopes: Vec<String>,
    ) -> String {
        let session_id = Ulid::new().to_string();
        let now = SystemTime::now();

        let session = Session {
            id: session_id.clone(),
            user_id,
            email,
            display_name,
            scopes,
            created_at: now,
            last_activity: now,
            expires_at: now + self.session_timeout,
        };

        self.sessions.insert(session_id.clone(), session).await;

        session_id
    }

    /// Get session by ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID
    ///
    /// # Returns
    ///
    /// Returns the session if found and valid
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session is not found or expired
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let session = self
            .sessions
            .get(session_id)
            .await
            .ok_or(AuthError::SessionNotFound)?;

        // Check if session is expired
        if session.is_expired() {
            self.sessions.invalidate(session_id).await;
            return Err(AuthError::SessionNotFound.into());
        }

        // Check if session is idle
        if session.is_idle(self.idle_timeout) {
            self.sessions.invalidate(session_id).await;
            return Err(AuthError::SessionNotFound.into());
        }

        Ok(session)
    }

    /// Update session activity
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if session is not found
    pub async fn touch_session(&self, session_id: &str) -> Result<()> {
        let mut session = self.get_session(session_id).await?;
        session.touch();
        self.sessions.insert(session_id.to_string(), session).await;
        Ok(())
    }

    /// Validate session and check binding
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID
    /// * `user_id` - User ID from current JWT (for binding check)
    ///
    /// # Returns
    ///
    /// Returns the session if valid and binding matches
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionValidationFailed` if binding check fails
    pub async fn validate_session(&self, session_id: &str, user_id: &str) -> Result<Session> {
        let session = self.get_session(session_id).await?;

        // Check session binding
        if self.enable_binding && session.user_id != user_id {
            return Err(AuthError::SessionValidationFailed(
                "Session user_id does not match JWT sub claim".to_string(),
            )
            .into());
        }

        // Touch session to update last activity
        self.touch_session(session_id).await?;

        Ok(session)
    }

    /// Invalidate (delete) a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to invalidate
    pub async fn invalidate_session(&self, session_id: &str) {
        self.sessions.invalidate(session_id).await;
    }

    /// Rotate session (create new session ID, preserve data)
    ///
    /// # Arguments
    ///
    /// * `old_session_id` - The old session ID
    ///
    /// # Returns
    ///
    /// Returns the new session ID
    ///
    /// # Errors
    ///
    /// Returns `AuthError::SessionNotFound` if old session is not found
    pub async fn rotate_session(&self, old_session_id: &str) -> Result<String> {
        let old_session = self.get_session(old_session_id).await?;

        // Create new session with same data
        let new_session_id = self
            .create_session(
                old_session.user_id,
                old_session.email,
                old_session.display_name,
                old_session.scopes,
            )
            .await;

        // Invalidate old session
        self.invalidate_session(old_session_id).await;

        Ok(new_session_id)
    }

    /// Get session count (for metrics)
    pub async fn session_count(&self) -> u64 {
        self.sessions.entry_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_is_expired() {
        let now = SystemTime::now();
        let session = Session {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            email: None,
            display_name: "Test".to_string(),
            scopes: vec![],
            created_at: now,
            last_activity: now,
            expires_at: now - Duration::from_secs(60),
        };

        assert!(session.is_expired());
    }

    #[test]
    fn test_session_is_not_expired() {
        let now = SystemTime::now();
        let session = Session {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            email: None,
            display_name: "Test".to_string(),
            scopes: vec![],
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::from_secs(60),
        };

        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_is_idle() {
        let now = SystemTime::now();
        let session = Session {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            email: None,
            display_name: "Test".to_string(),
            scopes: vec![],
            created_at: now,
            last_activity: now - Duration::from_secs(3600),
            expires_at: now + Duration::from_secs(60),
        };

        assert!(session.is_idle(Duration::from_secs(1800)));
    }

    #[test]
    fn test_session_has_scope() {
        let session = Session {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            email: None,
            display_name: "Test".to_string(),
            scopes: vec!["xzepr:read".to_string(), "xzepr:write".to_string()],
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
        };

        assert!(session.has_scope("xzepr:read"));
        assert!(session.has_scope("xzepr:write"));
        assert!(!session.has_scope("xzepr:admin"));
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let manager =
            SessionManager::new(Duration::from_secs(3600), Duration::from_secs(1800), true);

        let session_id = manager
            .create_session(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                "Test User".to_string(),
                vec!["xzepr:read".to_string()],
            )
            .await;

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.user_id, "user123");
        assert_eq!(session.display_name, "Test User");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let manager =
            SessionManager::new(Duration::from_secs(3600), Duration::from_secs(1800), true);

        let result = manager.get_session("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_session_with_binding() {
        let manager =
            SessionManager::new(Duration::from_secs(3600), Duration::from_secs(1800), true);

        let session_id = manager
            .create_session("user123".to_string(), None, "Test User".to_string(), vec![])
            .await;

        // Valid binding
        let result = manager.validate_session(&session_id, "user123").await;
        assert!(result.is_ok());

        // Invalid binding
        let result = manager.validate_session(&session_id, "other_user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_session_without_binding() {
        let manager = SessionManager::new(
            Duration::from_secs(3600),
            Duration::from_secs(1800),
            false, // Binding disabled
        );

        let session_id = manager
            .create_session("user123".to_string(), None, "Test User".to_string(), vec![])
            .await;

        // Should succeed even with different user_id
        let result = manager.validate_session(&session_id, "other_user").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalidate_session() {
        let manager =
            SessionManager::new(Duration::from_secs(3600), Duration::from_secs(1800), true);

        let session_id = manager
            .create_session("user123".to_string(), None, "Test User".to_string(), vec![])
            .await;

        manager.invalidate_session(&session_id).await;

        let result = manager.get_session(&session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rotate_session() {
        let manager =
            SessionManager::new(Duration::from_secs(3600), Duration::from_secs(1800), true);

        let old_session_id = manager
            .create_session(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                "Test User".to_string(),
                vec!["xzepr:read".to_string()],
            )
            .await;

        let new_session_id = manager.rotate_session(&old_session_id).await.unwrap();

        // Old session should be gone
        assert!(manager.get_session(&old_session_id).await.is_err());

        // New session should exist with same data
        let new_session = manager.get_session(&new_session_id).await.unwrap();
        assert_eq!(new_session.user_id, "user123");
        assert_eq!(new_session.email, Some("user@example.com".to_string()));
    }
}
