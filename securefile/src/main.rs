mod file;
mod user;
// use file::admin_file_management;
mod access_control;

use rpassword::read_password;
use sqlx::{mysql::MySqlPool, Row};
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database connection setup
    let database_url = "mysql://root:123%40Rohith@localhost/Files"; //your password and database name i forgot mysql123
    let pool = MySqlPool::connect(database_url).await?;

    // Login logic
    let email = get_input("Enter email: ");
    print!("Enter password: ");
    io::stdout().flush()?; // Flush prompt to stdout

    let password = read_password()?; // Secure password input
    let file_lock_manager = Arc::new(file::FileLockManager::new());

    match user::authenticate_user(&pool, &email, &password).await {
        Ok(role) => {
            println!("Login successful!");

            match role {
                user::UserRole::Admin => admin_menu(&pool).await?,
                user::UserRole::Manager => manager_menu(&pool, file_lock_manager, &email).await?, //manager_menu(&pool, &email).await?,
                user::UserRole::Director => manager_menu(&pool, file_lock_manager, &email).await?, //director_menu(&pool, &email).await?,
                user::UserRole::Developer => developer_menu(&pool, &email, role).await?,
            }
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}

async fn admin_menu(pool: &MySqlPool) -> Result<(), Box<dyn std::error::Error>> {
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
            "1" => user::user_management_menu(pool).await?,
            "2" => file::admin_file_management(pool).await,
            "3" => access_control::manage_access(pool).await?,
            _ => println!("Invalid option! Please try again."),
        }
    }
    Ok(())
}

async fn manager_menu(
    pool: &MySqlPool,
    file_lock_manager: Arc<file::FileLockManager>,
    email: &str,
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
                if file::check_if_file_locked(pool, file_id).await? {
                    println!("The file is currently locked. Please try again later.");
                    continue;
                }

                file::decrypt_and_edit_file(
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

async fn director_menu(
    pool: &MySqlPool,
    file_lock_manager: Arc<file::FileLockManager>,
    email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("Director Menu:");
        let choice = get_input("Do you want to access files? (yes/no): ");
        if choice.to_lowercase() != "yes" {
            println!("Goodbye!");
            return Ok(());
        }

        let filename = get_input("Enter file name: ");
        let file_id = fetch_file_id(pool, &filename).await?;
        println!("Access granted to the file.");

        let edit_choice = get_input("Do you want to edit the file? (yes/no): ");
        if edit_choice.to_lowercase() == "yes" {
            // Check if the file is locked in the database
            if file::check_if_file_locked(pool, file_id).await? {
                println!("The file is currently locked. Please try again later.");
                continue;
            }
            file::decrypt_and_edit_file(
                pool,
                file_id.try_into().unwrap(),
                file_lock_manager.clone(),
            )
            .await?;
        } else {
            decrypt_and_download_file(pool, file_id.try_into().unwrap()).await?;
        }
    }
    Ok(())
}

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

async fn developer_menu(
    pool: &MySqlPool,
    _email: &str,
    role: user::UserRole,
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

async fn check_access(
    pool: &MySqlPool,
    file_id: i32,
    role: &user::UserRole,
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

async fn decrypt_and_download_file(pool: &MySqlPool, _file_id: i32) -> Result<(), Box<dyn Error>> {
    // Query to get file information
    let row = sqlx::query("SELECT file_id, file_path, encrypted_key FROM Files WHERE file_id = ?")
        .bind(_file_id)
        .fetch_one(pool)
        .await?;

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
    let decrypted_content = file::decrypt_file(&file_path, &encrypted_key).await?;
    println!("Decrypted content: {}", decrypted_content);

    // Simulate downloading the decrypted file
    println!("File decrypted and downloaded successfully!");

    Ok(())
}

// Helper function to get user input
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
