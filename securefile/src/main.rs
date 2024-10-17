// fn main() {
//     println!("Welcome to Secure File Management!");
// }


use bcrypt::verify;
use sqlx::{mysql::MySqlPool, Row};
use rpassword::read_password;
// use bcrypt::verify;
use std::io::{self, Write};

// // Wrapping async main with Tokio runtime
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Update this with your database URL
//     let database_url = "mysql://root:123@Rohith@localhost/Files";
//     let pool = MySqlPool::connect(database_url).await?;

//     // Get user input for email and password
//     let email = get_input("Enter email: ");
//     print!("Enter password: ");
//     io::stdout().flush()?;  // Ensure prompt is printed before input

//     let password = read_password()?;  // Secure password input

//     // Authenticate user
//     match authenticate_user(&pool, &email, &password).await {
//         Ok(_) => println!("Login successful!"),
//         Err(e) => println!("{}", e),
//     }

//     Ok(())
// }

// // Helper function to get input from the user
// fn get_input(prompt: &str) -> String {
//     print!("{}", prompt);
//     io::stdout().flush().unwrap();

//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     input.trim().to_string()
// }

// // Function to authenticate the user with email and password
// async fn authenticate_user(
//     pool: &MySqlPool,
//     email: &str,
//     password: &str,
// ) -> Result<(), String> {
//     // Query the user from the database by email
//     let row = sqlx::query("SELECT password_hash, is_blocked FROM Users WHERE email = ?")
//         .bind(email)
//         .fetch_optional(pool)
//         .await
//         .map_err(|_| "Database query failed.")?;

//     // print!("{:#?}",row);

//     // Check if user exists
//     let row = match row {
//         Some(r) => r,
//         None => return Err("User not found.".to_string()),
//     };

//     // Check if user is blocked
//     if row.get::<bool, _>("is_blocked") {
//         return Err("User is blocked. Please contact admin.".to_string());
//     }

//     // Verify the password
//     let stored_hash: String = row.get("password_hash");
//     // print!("{}",password);
//     // if verify(password, &stored_hash).map_err(|_| "Error verifying password.")? 
//     if password == stored_hash{
//         Ok(())
//     } else {
//         Err("Incorrect password.".to_string())
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Update this with your database URL
    let database_url = "mysql://root:123@Rohith@localhost/Files";
    let pool = MySqlPool::connect(database_url).await?;

    // Get user input for email and password
    let email = get_input("Enter email: ");
    print!("Enter password: ");
    io::stdout().flush()?;  // Ensure prompt is printed before input

    let password = read_password()?;  // Secure password input

    // Authenticate user
    match authenticate_user(&pool, &email, &password).await {
        Ok(role) => {
            println!("Login successful!");

            // Check if the role is admin
            if role == "admin" {
                loop {
                    let add_user = get_input("Do you want to add a new user? (yes/no): ");
                    if add_user.to_lowercase() != "yes" {
                        break; // Exit the loop if the answer is not 'yes'
                    }

                    add_user_to_db(&pool).await?;
                }
            }
        },
        Err(e) => println!("{}", e),
    }

    Ok(())
}

// Function to authenticate the user with email and password
async fn authenticate_user(
    pool: &MySqlPool,
    email: &str,
    password: &str,
) -> Result<String, String> {
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
        let role: String = row.get("role");
        return Ok(role);
    } else {
        return Err("Incorrect password.".to_string());
    }
}

// Function to add a user to the database
async fn add_user_to_db(pool: &MySqlPool) -> Result<(), String> {
    // Get user information from the admin
    let username = get_input("Enter new username: ");
    let password = get_input("Enter new password: ");
    let role = get_input("Enter role (admin/developer/manager/director): ");

    // Check if role is valid
    if !["admin", "developer", "manager", "director"].contains(&role.as_str()) {
        return Err("Invalid role.".to_string());
    }

    // Hash the password before storing (replace this with bcrypt)
    // let password_hash = password; // For demonstration, store the password directly
    let password_hash = bcrypt::hash(&password, 4).map_err(|_| "Error hashing password.")?;


    // Insert the new user into the database
    // sqlx::query("INSERT INTO Users (username, password_hash, role) VALUES (?, ?, ?)")
    //     .bind(username)
    //     .bind(password_hash)
    //     .bind(role)
    //     .execute(pool)
    //     .await
    //     .map_err(|_| "Failed to add user to the database.")?;

    // println!("User added successfully!");

    // Ok(())
    // Insert the new user into the database
    let result = sqlx::query("INSERT INTO Users (email, password_hash, role) VALUES (?, ?, ?)")
        .bind(username)
        .bind(password_hash)
        .bind(role)
        .execute(pool)
        .await;

    match result {
        Ok(_) => {
            println!("User added successfully!");
            Ok(())
        },
        Err(e) => {
            println!("Error adding user to the database: {:?}", e);
            Err("Failed to add user to the database.".to_string())
        }
    }
}

// Function to get input from the user
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

