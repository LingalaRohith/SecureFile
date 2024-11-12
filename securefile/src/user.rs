// // This module handles user registration, authentication, role management, and blocking logic.

// pub struct User {
//     pub user_id: u32,
//     pub username: String,
//     pub password_hash: String,
//     pub role: UserRole,
//     pub failed_attempts: u32,
//     pub is_blocked: bool,
//     pub created_at: chrono::NaiveDateTime,
// }

// pub enum UserRole {
//     Admin,
//     Developer,
//     Manager,
//     Director,
// }

// // impl User {
// //     /// Registers a new user
// //     pub fn register(username: &str, password: &str, role: UserRole) -> Result<User, String> {
// //         // 1. Check if the username exists in the database
// //         if User::exists(username)? {
// //             return Err("Username already exists".to_string());
// //         }

// //         // 2. Hash the password (bcrypt can be used)
// //         let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

// //         // 3. Insert new user into the database
// //         let new_user = User {
// //             user_id: 0, // This will be auto-incremented in the DB
// //             username: username.to_string(),
// //             password_hash,
// //             role,
// //             failed_attempts: 0,
// //             is_blocked: false,
// //             created_at: chrono::Utc::now().naive_utc(),
// //         };

// //         // Insert into DB (this is pseudo-code, Diesel can be used)
// //         User::insert_into_db(new_user)?;

// //         Ok(new_user)
// //     }

// //     /// Authenticates a user with the provided credentials
// //     pub fn authenticate(username: &str, password: &str) -> Result<User, String> {
// //         // 1. Retrieve user from the database
// //         let user = User::find_by_username(username)?;

// //         // 2. Check if the user is blocked
// //         if user.is_blocked {
// //             return Err("User is blocked".to_string());
// //         }

// //         // 3. Verify the password
// //         if !bcrypt::verify(password, &user.password_hash).unwrap() {
// //             // Increment failed attempts and possibly block the user
// //             User::increment_failed_attempts(user.user_id)?;
// //             return Err("Invalid password".to_string());
// //         }

// //         // Reset failed attempts after successful login
// //         User::reset_failed_attempts(user.user_id)?;

// //         Ok(user)
// //     }

// //     /// Block the user after exceeding the allowed number of failed attempts
// //     pub fn block_user(user_id: u32) {
// //         // Set the user's `is_blocked` flag to true in the database
// //         // SQL: UPDATE Users SET is_blocked = TRUE WHERE user_id = user_id;
// //     }

// //     /// Helper function to check if a username exists
// //     fn exists(username: &str) -> Result<bool, String> {
// //         // Query the database to check if the username exists
// //         // SQL: SELECT COUNT(*) FROM Users WHERE username = username;
// //     }
// // }

// use sqlx::mysql::MySqlPool;
// use std::io::{self, Write};

// pub async fn add_user_to_db(pool: &MySqlPool) -> Result<(), String> {
//     let email = get_input("Enter new user email: ");
//     let password = get_input("Enter new user password: ");
//     let role = get_input("Enter role (admin/developer/manager/director): ");

//     if !["admin", "developer", "manager", "director"].contains(&role.as_str()) {
//         return Err("Invalid role.".to_string());
//     }

//     let password_hash = bcrypt::hash(&password, 4).map_err(|_| "Error hashing password.")?;
//     let result = sqlx::query("INSERT INTO Users (email, password_hash, role) VALUES (?, ?, ?)")
//         .bind(email)
//         .bind(password_hash)
//         .bind(role)
//         .execute(pool)
//         .await;

//     match result {
//         Ok(_) => {
//             println!("User added successfully!");
//             Ok(())
//         }
//         Err(e) => {
//             println!("Error adding user: {:?}", e);
//             Err("Failed to add user.".to_string())
//         }
//     }
// }

// fn get_input(prompt: &str) -> String {
//     print!("{}", prompt);
//     io::stdout().flush().unwrap();

//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     input.trim().to_string()
// }

use bcrypt::{hash, verify};
use sqlx::{mysql::MySqlPool, Row};
use std::io::{self, Write};
use std::fmt;

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub enum UserRole {
    Admin,
    Developer,
    Manager,
    Director,
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
