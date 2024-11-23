use secure_file_management::user::{self}; // Import `user` and `UserRole`
use secure_file_management::menu; 

use rpassword::read_password;
use sqlx::{mysql::MySqlPool};
// use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;

/// Entry point of the application. Manages user login and routes to respective menus based on user role.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database connection setup
    let database_url = "mysql://root:123%40Rohith@localhost:3306/Files"; //your password and database name i forgot mysql123
    let pool = MySqlPool::connect(database_url).await?;

    // Login logic
    let email = get_input("Enter email: ");
    print!("Enter password: ");
    io::stdout().flush()?; // Flush prompt to stdout

    let password = read_password()?; // Secure password input
    let file_lock_manager = Arc::new(secure_file_management::file::FileLockManager::new());

    match user::authenticate_user(&pool, &email, &password).await {
        Ok(role) => {
            println!("Login successful!");

            match role {
                user::UserRole::Admin => menu::admin_menu(&pool).await?,
                user::UserRole::Manager => menu::manager_menu(&pool, file_lock_manager, &email).await?, //manager_menu(&pool, &email).await?,
                user::UserRole::Director => menu::director_menu(&pool, file_lock_manager, &email).await?, //director_menu(&pool, &email).await?,
                user::UserRole::Developer => menu::developer_menu(&pool, &email, role).await?,
            }
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}

fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

