// This module handles user registration, authentication, role management, and blocking logic.

pub struct User {
    pub user_id: u32,
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
    pub failed_attempts: u32,
    pub is_blocked: bool,
    pub created_at: chrono::NaiveDateTime,
}

pub enum UserRole {
    Admin,
    Developer,
    Manager,
    Director,
}

impl User {
    /// Registers a new user
    pub fn register(username: &str, password: &str, role: UserRole) -> Result<User, String> {
        // 1. Check if the username exists in the database
        if User::exists(username)? {
            return Err("Username already exists".to_string());
        }

        // 2. Hash the password (bcrypt can be used)
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

        // 3. Insert new user into the database
        let new_user = User {
            user_id: 0, // This will be auto-incremented in the DB
            username: username.to_string(),
            password_hash,
            role,
            failed_attempts: 0,
            is_blocked: false,
            created_at: chrono::Utc::now().naive_utc(),
        };

        // Insert into DB (this is pseudo-code, Diesel can be used)
        User::insert_into_db(new_user)?;

        Ok(new_user)
    }

    /// Authenticates a user with the provided credentials
    pub fn authenticate(username: &str, password: &str) -> Result<User, String> {
        // 1. Retrieve user from the database
        let user = User::find_by_username(username)?;

        // 2. Check if the user is blocked
        if user.is_blocked {
            return Err("User is blocked".to_string());
        }

        // 3. Verify the password
        if !bcrypt::verify(password, &user.password_hash).unwrap() {
            // Increment failed attempts and possibly block the user
            User::increment_failed_attempts(user.user_id)?;
            return Err("Invalid password".to_string());
        }

        // Reset failed attempts after successful login
        User::reset_failed_attempts(user.user_id)?;

        Ok(user)
    }

    /// Block the user after exceeding the allowed number of failed attempts
    pub fn block_user(user_id: u32) {
        // Set the user's `is_blocked` flag to true in the database
        // SQL: UPDATE Users SET is_blocked = TRUE WHERE user_id = user_id;
    }

    /// Helper function to check if a username exists
    fn exists(username: &str) -> Result<bool, String> {
        // Query the database to check if the username exists
        // SQL: SELECT COUNT(*) FROM Users WHERE username = username;
    }
}
