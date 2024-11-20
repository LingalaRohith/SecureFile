// Manages user access based on roles, file priority, and master key.
use sqlx::{query, MySqlPool};
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug)]
pub enum UserRole {
    Admin,
    Developer,
    Manager,
    Director,
}

/// Converts a string input into a valid `UserRole` enum.
///
/// The `FromStr` trait implementation enables parsing a string into one of the four `UserRole`
/// variants: Admin, Developer, Manager, or Director. The string comparison is case-insensitive.
///
/// # Arguments
///
/// * `s` - The string to parse into a `UserRole` variant.
///
/// # Returns
///
/// Returns a `Result` containing either a valid `UserRole` or an error message if the string cannot
/// be parsed into a known role.

impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "developer" => Ok(UserRole::Developer),
            "manager" => Ok(UserRole::Manager),
            "director" => Ok(UserRole::Director),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

/// Manages user access control based on roles, file priority, and time restrictions.
///
/// This function interacts with the MySQL database to insert a record into the `UserAccessControl`
/// table, which determines whether a user has access to a specific file. The decision is based on
/// the required role, whether access is granted, and if there are any time restrictions in place.
///
/// The access control mechanism relies on the following parameters:
/// 1. `file_id`: The ID of the file to which access is being controlled.
/// 2. `access_granted`: A boolean value indicating whether the user has access to the file.
/// 3. `role_required`: The role required to access the file. The possible roles are Admin, Developer, Manager, and Director.
/// 4. `time_restricted`: A boolean value indicating whether access to the file is restricted by time (e.g., office hours only).
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL connection pool (`MySqlPool`) used to interact with the database.
///
/// # Returns
///
/// Returns `Result<(), Box<dyn std::error::Error>>`, indicating whether the user access control
/// record was successfully inserted into the database or an error occurred.
///
/// # Example
/// ```rust
/// let pool: MySqlPool = // your pool initialization here
/// manage_access(&pool).await.unwrap();
/// ```
///
/// # How it works:
/// 1. The user is prompted to input a file ID, access permission (yes/no), required role, and time restriction status.
/// 2. The provided input is validated, and the access control information is inserted into the `UserAccessControl` table.
/// 3. If the operation succeeds, a confirmation message is shown; otherwise, an error message is returned.

pub async fn manage_access(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Managing access control...");

    let file_id: u32 = get_input("Enter file ID: ")
        .parse()
        .map_err(|_| "Invalid file ID")?;

    let access_granted = get_input("Access granted? (yes/no): ").to_lowercase() == "yes";

    let role_required = loop {
        let role_input = get_input("Enter required role (admin, developer, manager, director): ");
        match role_input.parse::<UserRole>() {
            Ok(role) => break role,
            Err(e) => println!("Error: {}", e),
        }
    };

    let time_restricted = get_input("Time restricted? (yes/no): ").to_lowercase() == "yes";

    // Execute the SQL query to insert access control record
    let result = sqlx::query("INSERT INTO UserAccessControl (file_id, access_granted, role_required, time_restricted) VALUES (?, ?, ?, ?)")
        .bind(file_id)
        .bind(access_granted)
        .bind(role_required.to_string())
        .bind(time_restricted)
        .execute(pool)
        .await;

    match result {
        Ok(_) => {
            println!("User added successfully!");
            return Ok(());
        }
        Err(e) => {
            println!("Error adding user: {:?}", e);
            return Err("Failed to add user to the database.".into());
        }
    }
}

// Helper function to get user input
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}


/// Converts a `UserRole` into a string for database storage.
///
/// The `ToString` trait implementation provides a string representation for each `UserRole` variant
/// to facilitate insertion into the database or other string-based operations.
///
/// # Returns
///
/// Returns the string representation of the `UserRole` variant (e.g., "admin", "developer", etc.).

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Developer => "developer".to_string(),
            UserRole::Manager => "manager".to_string(),
            UserRole::Director => "director".to_string(),
        }
    }
}
