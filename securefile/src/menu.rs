// mod file;
// mod user;
// // use file::admin_file_management;
// mod access_control;

use crate::file::{admin_file_management, check_if_file_locked, decrypt_and_edit_file, decrypt_file, FileLockManager};
use crate::user::{UserRole, user_management_menu};
use crate::access_control::manage_access;


// use rpassword::read_password;
use sqlx::{mysql::MySqlPool, Row};
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;


/// Displays the Admin menu and handles corresponding functionality.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
///
/// # Errors
///
/// Returns an error if database operations or user input fail.

pub async fn admin_menu(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("\nAdmin Menu");
        println!("1. User Management");
        println!("2. File Management");
        println!("3. Access Control");
        println!("Press Enter to quit.");

        let choice = get_input("> ");
        if choice.is_empty() {
            println!("Exiting...");
            break;
        }

        match choice.as_str() {
            "1" => user_management_menu(pool).await?,
            "2" => admin_file_management(pool).await,
            "3" => manage_access(pool).await?,
            _ => println!("Invalid option! Please try again."),
        }
    }
    Ok(())
}


/// Displays the Manager menu and provides file access based on priority.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
/// * `file_lock_manager` - A shared reference to the file lock manager.
/// * `email` - The email of the logged-in user.
///
/// # Errors
///
/// Returns an error if database operations fail or user input is invalid.

pub async fn manager_menu(
    pool: &MySqlPool,
    file_lock_manager: Arc<FileLockManager>,
    _email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Manager Menu:");
        let choice = get_input("Do you want to access files? (yes/no): ");
        if choice.to_lowercase() != "yes" {
            println!("Goodbye!");
            return Ok(());
        }

        let filename = get_input("Enter file name: ");
        let file_id = fetch_file_id(pool, &filename).await?;
        let file_priority = fetch_file_priority(pool, file_id.try_into().unwrap()).await?;

        if file_priority <= 3 {
            println!(
                "Access granted to the file with priority {}.",
                file_priority
            );
            let edit_choice = get_input("Do you want to edit the file? (yes/no): ");
            if edit_choice.to_lowercase() == "yes" {
                // Check if the file is locked in the database
                if check_if_file_locked(pool, file_id).await? {
                    println!("The file is currently locked. Please try again later.");
                    continue;
                }

                decrypt_and_edit_file(
                    pool,
                    file_id.try_into().unwrap(),
                    file_lock_manager.clone(),
                )
                .await?;
            }
        } else {
            let choice = get_input("Access denied. You can only edit files with priority <= 3. View all the files Do you want to get read only file? (yes/no):");
            if choice.to_lowercase() == "yes" {
                decrypt_and_download_file(pool, file_id.try_into().unwrap()).await?;
            }
        }
        println!("Press Enter to quit.");
        let choice = get_input("> ");
        if choice.is_empty() {
            println!("Exiting...");
            break;
        }
    }
    Ok(())
}

/// Handles the menu for the Director role, allowing them to access and edit files.
///
/// # Arguments
/// * `pool` - A reference to the MySQL database connection pool.
/// * `file_lock_manager` - A shared instance of the file lock manager to handle file locking.
/// * `email` - The email address of the logged-in director.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Returns an `Ok` result if the operation completes successfully, otherwise returns an error.


pub async fn director_menu(
    pool: &MySqlPool,
    file_lock_manager: Arc<FileLockManager>,
    _email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Director Menu:");
        let choice = get_input("Do you want to access files? (yes/no): ");
        if choice.to_lowercase() != "yes" {
            println!("Goodbye!");
            break;
            // return Ok(());
        }

        let filename = get_input("Enter file name: ");
        let file_id = fetch_file_id(pool, &filename).await?;
        println!("Access granted to the file.");

        let edit_choice = get_input("Do you want to edit the file? (yes/no): ");
        if edit_choice.to_lowercase() == "yes" {
            // Check if the file is locked in the database
            if check_if_file_locked(pool, file_id).await? {
                println!("The file is currently locked. Please try again later.");
                continue;
            }
            decrypt_and_edit_file(
                pool,
                file_id.try_into().unwrap(),
                file_lock_manager.clone(),
            )
            .await?;
        } else {
            decrypt_and_download_file(pool, file_id.try_into().unwrap()).await?;
        }
    }
    #[warn(unreachable_code)]
    Ok(())
}

/// Retrieves the file priority from the database.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL database connection pool.
/// * `file_id` - The ID of the file.
///
/// # Returns
///
/// The priority level of the file.
///
/// # Errors
///
/// Returns an error if the file is not found or a database error occurs.

async fn fetch_file_priority(
    pool: &MySqlPool,
    file_id: u32,
) -> Result<i32, Box<dyn std::error::Error>> {
    // let row = sqlx::query!("SELECT priority_level FROM Files WHERE file_id = ?", file_id)
    //     .fetch_one(pool)
    //     .await?;

    let row = sqlx::query("SELECT priority_level FROM Files WHERE file_id = ?")
        .bind(file_id)
        .fetch_optional(pool)
        .await?
        .ok_or("File not found.")?;

    // Retrieve the priority level from the row
    let priority_level: i32 = row.try_get("priority_level")?;

    Ok(priority_level)

    // Ok(row.priority_level)
}

/// Handles the menu for the Developer role, allowing them to access files if permitted.
///
/// # Arguments
/// * `pool` - A reference to the MySQL database connection pool.
/// * `_email` - The email address of the logged-in developer.
/// * `role` - The role of the user (Developer).
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Returns an `Ok` result if the operation completes successfully, otherwise returns an error.


pub async fn developer_menu(
    pool: &MySqlPool,
    _email: &str,
    role: UserRole,
) -> Result<(), Box<dyn std::error::Error>> {
    let choice = get_input("Are you here to access files? (yes/no): ");
    if choice.to_lowercase() != "yes" {
        println!("Goodbye!");
        return Ok(());
    }

    let filename = get_input("Enter file name: ");
    let file_id = fetch_file_id(pool, &filename).await?;

    // Check if access is granted for the user's role
    let access_granted = check_access(pool, file_id.try_into().unwrap(), &role).await?;

    if access_granted {
        println!("Access granted. Decrypting and downloading the file...");
        decrypt_and_download_file(pool, file_id.try_into().unwrap()).await?;
        // Check if the role is Manager or Director and prompt for edit option
        //  if matches!(role, user::UserRole::Manager | user::UserRole::Director) {
        //  file::edit_file(pool, file_id.try_into().unwrap()).await?;
        //  }
    } else {
        println!("Access denied. You do not have the required permissions.");
    }

    Ok(())
}

/// Fetches the file ID for a given file name from the database.
///
/// # Arguments
/// * `pool` - A reference to the MySQL database connection pool.
/// * `filename` - The name of the file for which to fetch the ID.
///
/// # Returns
/// * `Result<i32, Box<dyn std::error::Error>>` - The ID of the file if found, or an error if not found.


async fn fetch_file_id(
    pool: &MySqlPool,
    filename: &str,
) -> Result<i32, Box<dyn std::error::Error>> {
    let row = sqlx::query("SELECT file_id FROM Files WHERE file_name = ?")
        .bind(filename)
        .fetch_optional(pool)
        .await?
        .ok_or("File not found.")?;

    Ok(row.get("file_id"))
}

/// Checks whether a user role has access to a file based on its ID.
///
/// # Arguments
/// * `pool` - A reference to the MySQL database connection pool.
/// * `file_id` - The ID of the file to check access for.
/// * `role` - The role of the user attempting to access the file.
///
/// # Returns
/// * `Result<bool, Box<dyn std::error::Error>>` - Returns `true` if access is granted, otherwise `false`.


async fn check_access(
    pool: &MySqlPool,
    file_id: i32,
    role: &UserRole,
) -> Result<bool, Box<dyn std::error::Error>> {
    let role_str = role.to_string();
    let row = sqlx::query(
        "SELECT access_granted FROM UserAccessControl 
         WHERE file_id = ? AND role_required = ?",
    )
    .bind(file_id)
    .bind(role_str)
    .fetch_optional(pool)
    .await?;

    Ok(row.map_or(false, |r| r.get::<bool, _>("access_granted")))
}

/// Decrypts a file and simulates its download.
///
/// # Arguments
/// * `pool` - A reference to the MySQL database connection pool.
/// * `_file_id` - The ID of the file to decrypt and download.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Returns an `Ok` result if the operation completes successfully, otherwise returns an error.

pub async fn decrypt_and_download_file(pool: &MySqlPool, _file_id: i32) -> Result<(), Box<dyn Error>> {
    // Query to get file information
    let row = sqlx::query("SELECT file_id, file_path, encrypted_key FROM Files WHERE file_id = ?")
        .bind(_file_id)
        .fetch_optional(pool)
        .await?
        .ok_or("Row not found.")?;

    // Extract file_id and file_path
    let _file_id: i32 = row.get("file_id");
    let file_path: String = row.get("file_path");

    // Extract encrypted_key, which is stored as binary data in a BLOB column
    let encrypted_key: Vec<u8> = row.get("encrypted_key");

    // Convert the binary key to a Base64-encoded string if needed, or directly use it as &[u8]
    // Note: decrypt_file function should be compatible with &[u8]
    println!(
        "Decrypting file from path: {} with key: {:?}...",
        file_path, encrypted_key
    );

    // Call the decrypt_file function, passing the file path and binary key directly
    let decrypted_content = decrypt_file(&file_path, &encrypted_key,Some("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR")).await?;
    println!("Decrypted content: {}", decrypted_content);

    // Simulate downloading the decrypted file
    println!("File decrypted and downloaded successfully!");

    Ok(())
}

/// Helper function to get user input from the terminal.
///
/// # Arguments
///
/// * `prompt` - The prompt message to display to the user.
///
/// # Returns
///
/// The input string entered by the user.

// Helper function to get user input
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
