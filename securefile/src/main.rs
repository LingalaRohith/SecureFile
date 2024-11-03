
mod user;
mod file;
// use file::admin_file_management;
mod access_control;

use std::io::{self, Write};
use rpassword::read_password;
use sqlx::{mysql::MySqlPool, Row};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_main().await
}
async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    // Database connection setup
    let database_url = "mysql://tarun:admin@localhost/securefiledb";
    let pool = MySqlPool::connect(database_url).await?;

    // Login logic
    let email = get_input("Enter email: ");
    print!("Enter password: ");
    io::stdout().flush()?;  // Flush prompt to stdout

    let password = read_password()?;  // Secure password input

    match user::authenticate_user(&pool, &email, &password).await {
        Ok(role) => {
            println!("Login successful!");

            match role {
                user::UserRole::Admin => admin_menu(&pool).await?,
                _ => handle_file_access(&pool, &email, role).await?,
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

async fn handle_file_access(
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
    let access_granted = check_access(pool, file_id.try_into().unwrap(), role).await?;

    if access_granted {
        println!("Access granted. Decrypting and downloading the file...");
        decrypt_and_download_file(pool, file_id.try_into().unwrap()).await?;
    } else {
        println!("Access denied. You do not have the required permissions.");
    }

    Ok(())
}

async fn fetch_file_id(pool: &MySqlPool, filename: &str) -> Result<i32, Box<dyn std::error::Error>> {
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
    role: user::UserRole,
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

async fn decrypt_and_download_file(
    pool: &MySqlPool,
    _file_id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let _file_name = String::new();


    let row = sqlx::query("SELECT file_id, file_path, encrypted_key FROM Files WHERE file_id = ?")
    .bind(_file_id)
    .fetch_one(pool)
    .await?;

let _file_id: i32 = row.get("file_id");  // Use i32 for INT columns
let file_path: String = row.get("file_path");
// let encryption_key: String = row.get("encrypted_key");
let encryption_key = String::from_utf8(row.get("encrypted_key"))
    .expect("Failed to convert encryption key to string");


    // Placeholder for decryption logic
    println!(
        "Decrypting file from path: {} with key: {}...",
        file_path, encryption_key
    );

    let decrypted_content = file::decrypt_file(&file_path, encryption_key).await?;
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

