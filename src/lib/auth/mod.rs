mod models;
mod password;
mod repository;
mod invitation;
pub mod middleware;

pub use models::{User, UserRole, Credentials, RegisterRequest, AuthResponse, UserResponse};
pub use password::{hash_password, verify_password};
pub use repository::UserRepository;
pub use invitation::{
    Invitation, InvitationStatus, InvitationWithInviter, InviterInfo,
    CreateInvitationRequest, InvitationRepository
};
pub use middleware::{require_admin, require_auth};

use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::VerifyError;
use sqlx::PgPool;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hash error: {0}")]
    PasswordHash(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<VerifyError> for AuthError {
    fn from(err: VerifyError) -> Self {
        match err {
            VerifyError::PasswordInvalid => AuthError::InvalidCredentials,
            _ => AuthError::PasswordHash(err.to_string()),
        }
    }
}

/// Backend for axum-login authentication
#[derive(Clone)]
pub struct Backend {
    db: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let repo = UserRepository::new(self.db.clone());

        // Find user by email
        let user = match repo.find_by_email(&creds.email).await? {
            Some(user) => user,
            None => return Ok(None),
        };

        // Verify password
        match verify_password(&creds.password, &user.password_hash) {
            Ok(true) => {
                // Update last login
                let _ = repo.update_last_login(user.id).await;
                Ok(Some(user))
            }
            Ok(false) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let repo = UserRepository::new(self.db.clone());
        repo.find_by_id(*user_id).await.map_err(Into::into)
    }
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Backend").finish()
    }
}

/// Type alias for authentication session
pub type AuthSession = axum_login::AuthSession<Backend>;
