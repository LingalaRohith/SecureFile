// // This module handles user registration, authentication, role management, and blocking logic.

use bcrypt::{hash, verify};
use sqlx::{mysql::MySqlPool, Row};
use std::io::{self, Write};
use std::fmt;


/// Represents the role of a user in the system.
#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub enum UserRole {
    /// Role with administrative privileges.
    Admin,
    /// Role for software developers.
    Developer,
    /// Role for managers with specific access rights.
    Manager,
    /// Role for directors with high-level access.
    Director,
    // Admin,
    // Developer,
    // Manager,
    // Director,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role_str = match self {
            UserRole::Admin => "admin",
            UserRole::Developer => "developer",
            UserRole::Manager => "manager",
            UserRole::Director => "director",
        };
        write!(f, "{}", role_str)
    }
}


/// Authenticates a user by verifying their email and password.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
/// * `email` - The email of the user attempting to log in.
/// * `password` - The plain-text password provided by the user.
///
/// # Returns
///
/// * `Ok(UserRole)` - If authentication is successful, returns the user's role.
/// * `Err(String)` - If authentication fails, returns an error message.
///
/// # Errors
///
/// This function returns an error if:
/// - The user is not found.
/// - The user is blocked.
/// - The provided password is incorrect.
/// - The database query fails.
// Function to authenticate the user with email and password
pub async fn authenticate_user(
    pool: &MySqlPool,
    email: &str,
    password: &str,
) -> Result<UserRole, String> {
    // Query the user from the database by email
    let row = sqlx::query("SELECT password_hash, is_blocked, role FROM Users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Database query failed.")?;

    // Check if user exists
    let row = match row {
        Some(r) => r,
        None => return Err("User not found.".to_string()),
    };

    // Check if user is blocked
    if row.get::<bool, _>("is_blocked") {
        return Err("User is blocked. Please contact admin.".to_string());
    }

    // Compare the plain text password
    let stored_password: String = row.get("password_hash");
    if password == stored_password || verify(password, &stored_password).unwrap_or(false){
        // Return the role upon successful login
        let _role: String = row.get("role");
        // return Ok(role);
        let role: String = row.get("role");
        match role.as_str() {
            "admin" => Ok(UserRole::Admin),
            "developer" => Ok(UserRole::Developer),
            "manager" => Ok(UserRole::Manager),
            "director" => Ok(UserRole::Director),
            _ => Err("Invalid role.".to_string()),
        }
    } else {
        return Err("Incorrect password.".to_string());
    }
   
}


/// Displays the user management menu, allowing administrators to add users.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
///
/// # Returns
///
/// * `Ok(())` - If the operation completes successfully.
/// * `Err(String)` - If an error occurs during user addition.
///
/// This function provides a loop to allow adding multiple users.

// User management menu
pub async fn user_management_menu(pool: &MySqlPool) -> Result<(), String> {
    loop {
        add_user_to_db(pool).await?;

        let choice = get_input("Do you want to add another user? (yes/no): ");
        if choice.to_lowercase() != "yes" {
            break;
        }
    }
    Ok(())
}

/// Adds a new user to the database.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
///
/// # Returns
///
/// * `Ok(())` - If the user is added successfully.
/// * `Err(String)` - If an error occurs during user addition.
///
/// This function prompts the user for an email, password, and role, and stores the new user in the database.

// Function to add a user to the database
pub async fn add_user_to_db(pool: &MySqlPool) -> Result<(), String> {
    let email = get_input("Enter new email: ");
    let password = get_input("Enter new password: ");
    let role = get_input("Enter role (admin/developer/manager/director): ");

    if !["admin", "developer", "manager", "director"].contains(&role.as_str()) {
        return Err("Invalid role.".to_string());
    }

    let password_hash = hash(&password, 4).map_err(|_| "Error hashing password.")?;

    let result = sqlx::query("INSERT INTO Users (email, password_hash, role) VALUES (?, ?, ?)")
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .execute(pool)
        .await;

    match result {
        Ok(_) => {
            println!("User added successfully!");
            Ok(())
        }
        Err(e) => {
            println!("Error adding user: {:?}", e);
            Err("Failed to add user to the database.".to_string())
        }
    }
}

// Helper function to get input from the user
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
