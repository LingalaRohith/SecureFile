// Manages user access based on roles, file priority, and master key.
use sqlx::{MySqlPool, query};
use std::io::{self, Write};
use std::str::FromStr;
pub struct AccessControl {
    pub file_id: u32,
    access_granted: bool,
    pub role_required: UserRole,
    pub time_restricted: bool,
}

#[derive(Debug)]
pub enum UserRole {
    Admin,
    Developer,
    Manager,
    Director,
}


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


pub async fn manage_access(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Managing access control...");

    let file_id: u32 = get_input("Enter file ID: ").parse()
        .map_err(|_| "Invalid file ID")?;

    let access_granted = get_input("Access granted? (yes/no): ")
        .to_lowercase() == "yes";

    let role_required = loop {
        let role_input = get_input("Enter required role (admin, developer, manager, director): ");
        match role_input.parse::<UserRole>() {
            Ok(role) => break role,
            Err(e) => println!("Error: {}", e),
        }
    };

    let time_restricted = get_input("Time restricted? (yes/no): ")
        .to_lowercase() == "yes";

    // Insert the new access control record into the UserAccessControl table
    query!(
        "INSERT INTO UserAccessControl (file_id, access_granted, role_required, time_restricted) 
         VALUES (?, ?, ?, ?)",
        file_id,
        access_granted,
        role_required.to_string(),
        time_restricted,
    )
    .execute(pool)
    .await?;

    println!("Access control entry added successfully.");
    Ok(())
}

// Helper function to get user input
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

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


