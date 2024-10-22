// // fn main() {
// //     println!("Welcome to Secure File Management!");
// // }


// use bcrypt::verify;
// use sqlx::{mysql::MySqlPool, Row};
// use rpassword::read_password;
// // use bcrypt::verify;
// use std::io::{self, Write};

// // // Wrapping async main with Tokio runtime
// // #[tokio::main]
// // async fn main() -> Result<(), Box<dyn std::error::Error>> {
// //     // Update this with your database URL
// //     let database_url = "mysql://root:123@Rohith@localhost/Files";
// //     let pool = MySqlPool::connect(database_url).await?;

// //     // Get user input for email and password
// //     let email = get_input("Enter email: ");
// //     print!("Enter password: ");
// //     io::stdout().flush()?;  // Ensure prompt is printed before input

// //     let password = read_password()?;  // Secure password input

// //     // Authenticate user
// //     match authenticate_user(&pool, &email, &password).await {
// //         Ok(_) => println!("Login successful!"),
// //         Err(e) => println!("{}", e),
// //     }

// //     Ok(())
// // }

// // // Helper function to get input from the user
// // fn get_input(prompt: &str) -> String {
// //     print!("{}", prompt);
// //     io::stdout().flush().unwrap();

// //     let mut input = String::new();
// //     io::stdin().read_line(&mut input).unwrap();
// //     input.trim().to_string()
// // }

// // // Function to authenticate the user with email and password
// // async fn authenticate_user(
// //     pool: &MySqlPool,
// //     email: &str,
// //     password: &str,
// // ) -> Result<(), String> {
// //     // Query the user from the database by email
// //     let row = sqlx::query("SELECT password_hash, is_blocked FROM Users WHERE email = ?")
// //         .bind(email)
// //         .fetch_optional(pool)
// //         .await
// //         .map_err(|_| "Database query failed.")?;

// //     // print!("{:#?}",row);

// //     // Check if user exists
// //     let row = match row {
// //         Some(r) => r,
// //         None => return Err("User not found.".to_string()),
// //     };

// //     // Check if user is blocked
// //     if row.get::<bool, _>("is_blocked") {
// //         return Err("User is blocked. Please contact admin.".to_string());
// //     }

// //     // Verify the password
// //     let stored_hash: String = row.get("password_hash");
// //     // print!("{}",password);
// //     // if verify(password, &stored_hash).map_err(|_| "Error verifying password.")? 
// //     if password == stored_hash{
// //         Ok(())
// //     } else {
// //         Err("Incorrect password.".to_string())
// //     }
// // }

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
//         Ok(role) => {
//             println!("Login successful!");

//             // Check if the role is admin
//             if role == "admin" {
//                 loop {
//                     let add_user = get_input("Do you want to add a new user? (yes/no): ");
//                     if add_user.to_lowercase() != "yes" {
//                         break; // Exit the loop if the answer is not 'yes'
//                     }

//                     add_user_to_db(&pool).await?;
//                 }
//             }
//         },
//         Err(e) => println!("{}", e),
//     }

//     Ok(())
// }

// // Function to authenticate the user with email and password
// async fn authenticate_user(
//     pool: &MySqlPool,
//     email: &str,
//     password: &str,
// ) -> Result<String, String> {
//     // Query the user from the database by email
//     let row = sqlx::query("SELECT password_hash, is_blocked, role FROM Users WHERE email = ?")
//         .bind(email)
//         .fetch_optional(pool)
//         .await
//         .map_err(|_| "Database query failed.")?;

//     // Check if user exists
//     let row = match row {
//         Some(r) => r,
//         None => return Err("User not found.".to_string()),
//     };

//     // Check if user is blocked
//     if row.get::<bool, _>("is_blocked") {
//         return Err("User is blocked. Please contact admin.".to_string());
//     }

//     // Compare the plain text password
//     let stored_password: String = row.get("password_hash");
//     if password == stored_password || verify(password, &stored_password).unwrap_or(false){
//         // Return the role upon successful login
//         let role: String = row.get("role");
//         return Ok(role);
//     } else {
//         return Err("Incorrect password.".to_string());
//     }
// }

// // Function to add a user to the database
// async fn add_user_to_db(pool: &MySqlPool) -> Result<(), String> {
//     // Get user information from the admin
//     let username = get_input("Enter new username: ");
//     let password = get_input("Enter new password: ");
//     let role = get_input("Enter role (admin/developer/manager/director): ");

//     // Check if role is valid
//     if !["admin", "developer", "manager", "director"].contains(&role.as_str()) {
//         return Err("Invalid role.".to_string());
//     }

//     // Hash the password before storing (replace this with bcrypt)
//     // let password_hash = password; // For demonstration, store the password directly
//     let password_hash = bcrypt::hash(&password, 4).map_err(|_| "Error hashing password.")?;


//     // Insert the new user into the database
//     // sqlx::query("INSERT INTO Users (username, password_hash, role) VALUES (?, ?, ?)")
//     //     .bind(username)
//     //     .bind(password_hash)
//     //     .bind(role)
//     //     .execute(pool)
//     //     .await
//     //     .map_err(|_| "Failed to add user to the database.")?;

//     // println!("User added successfully!");

//     // Ok(())
//     // Insert the new user into the database
//     let result = sqlx::query("INSERT INTO Users (email, password_hash, role) VALUES (?, ?, ?)")
//         .bind(username)
//         .bind(password_hash)
//         .bind(role)
//         .execute(pool)
//         .await;

//     match result {
//         Ok(_) => {
//             println!("User added successfully!");
//             Ok(())
//         },
//         Err(e) => {
//             println!("Error adding user to the database: {:?}", e);
//             Err("Failed to add user to the database.".to_string())
//         }
//     }
// }

// // Function to get input from the user
// fn get_input(prompt: &str) -> String {
//     print!("{}", prompt);
//     io::stdout().flush().unwrap();

//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     input.trim().to_string()
// }

mod user;
mod file;
// use file::admin_file_management;
mod access_control;

// use sqlx::mysql::MySqlPool;
// use std::io::{self, Write};

// use sqlx::mysql::MySqlPool;
use std::io::{self, Write};
use rpassword::read_password;
use sqlx::{mysql::MySqlPool, Row};


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Database connection setup
//     let database_url = "mysql://root:123@Rohith@localhost/Files";
//     let pool = MySqlPool::connect(database_url).await?;

//     // Login logic
//     let email = get_input("Enter email: ");
//     print!("Enter password: ");
//     io::stdout().flush()?;  // Flush prompt to stdout

//     let password = rpassword::read_password()?;  // Secure password input

//     match user::authenticate_user(&pool, &email, &password).await {
//         Ok(role) => {
//             println!("Login successful!");

//             if role == user::UserRole::Admin {
//                 loop {
//                     println!("\nMenu");
//                     println!("1. User Management");
//                     println!("2. File Management");
//                     println!("3. Access Control");
//                     println!("Press Enter to quit.");

//                     let choice = get_input("> ");
//                     if choice.is_empty() {
//                         println!("Exiting...");
//                         break;
//                     }

//                     match choice.as_str() {
//                         "1" => user::user_management_menu(&pool).await?,
//                         "2" => file::admin_file_management(&pool).await,
//                         "3" => access_control::manage_access(&pool).await?,
//                         _ => println!("Invalid option! Please try again."),
//                     }
//                 }
//             }
//         }
//         Err(e) => println!("{}", e),
//     }

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database connection setup
    let database_url = "mysql://root:123@Rohith@localhost/Files";
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
    let mut file_name = String::new();
// println!("Enter file name: ");
// std::io::stdin().read_line(&mut file_name).expect("Failed to read input");
// Remove any trailing newline characters
// let file_name = file_name.trim();

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

