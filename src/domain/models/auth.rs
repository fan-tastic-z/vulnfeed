use crate::domain::models::admin_user::{AdminUserPassword, AdminUsername};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoginRequest {
    pub username: AdminUsername,
    pub password: AdminUserPassword,
}

impl LoginRequest {
    pub fn new(username: AdminUsername, password: AdminUserPassword) -> Self {
        Self { username, password }
    }
}
