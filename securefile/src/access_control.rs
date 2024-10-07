// Manages user access based on roles, file priority, and master key.

pub struct AccessControl {
    pub user_id: u32,
    pub file_id: u32,
    access_granted: bool,
    pub role_required: UserRole,
    pub time_restricted: bool,
}

impl AccessControl {
    /// Checks if the user has access to a file based on their role and file priority
    pub fn check_access(user_id: u32, file_priority: u8) -> Result<bool, String> {
        let user = User::find_by_id(user_id)?;

        match user.role {
            UserRole::Admin => Ok(true),  // Admin has access to everything
            UserRole::Manager => Ok(file_priority <= 3),  // Managers can access priority 3 and below
            UserRole::Director => Ok(file_priority <= 4), // Directors can access priority 4 and below
            UserRole::Developer => Ok(file_priority <= 1),  // Developers have minimal access
        }
    }
}

