// use mysql::prelude::*;
// use mysql::*;
// use std::error::Error;
// use std::fs;

// fn main() -> Result<(), Box<dyn Error>> {
//     // set up database url
//     let url = "mysql://root:mysql123@localhost:3306/Files"; //

//     // create connection pool
//     let pool = Pool::new(url)?;
//     let mut conn = pool.get_conn()?;

//     // load schema.sql
//     let schema = fs::read_to_string("./schema.sql")?; //
//                                                       // execute schema.sql
//     conn.query_drop(schema)?;

//     println!("Schema executed successfully!");

//     Ok(())
// }
// -------------
// use sqlx::mysql::MySqlPool;
// use std::error::Error;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Database connection setup
//     let database_url = "mysql://root:mysql123@localhost/Files";
//     let pool = MySqlPool::connect(database_url).await?;

//     println!("Database connected successfully!");

//     Ok(())
// }
// ---------------
// use sqlx::{mysql::MySqlPool, Row};
// use std::error::Error;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Database connection setup
//     let database_url = "mysql://root:mysql123@localhost/Files";
//     let pool = MySqlPool::connect(database_url).await?;

//     // Fetch file information
//     let file_name = "example_file.txt"; //
//     let row =
//         sqlx::query("SELECT file_id, file_path, encrypted_key FROM Files WHERE file_name = ?")
//             .bind(file_name)
//             .fetch_optional(&pool)
//             .await?
//             .ok_or("File not found.")?;

//     let file_id: i32 = row.get("file_id");
//     let file_path: String = row.get("file_path");
//     let encrypted_key: Vec<u8> = row.get("encrypted_key");

//     println!(
//         "File ID: {}, Path: {}, Encrypted Key: {:?}",
//         file_id, file_path, encrypted_key
//     );

//     Ok(())
// }
// // -----------
// use sqlx::{mysql::MySqlPool, Row};
// use std::error::Error;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Database connection setup
//     let database_url = "mysql://root:mysql123@localhost/Files";
//     let pool = MySqlPool::connect(database_url).await?;

//     // Check access control
//     let file_id = 1;
//     let role = "Admin";
//     let row = sqlx::query(
//         "SELECT access_granted FROM UserAccessControl WHERE file_id = ? AND role_required = ?",
//     )
//     .bind(file_id)
//     .bind(role)
//     .fetch_optional(&pool)
//     .await?;

//     let access_granted = row.map_or(false, |r| r.get::<bool, _>("access_granted"));
//     println!("Access granted: {}", access_granted);

//     Ok(())
// }
// ------------
use sqlx::{mysql::MySqlPool, Row};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Database connection setup
    let database_url = "mysql://root:mysql123@localhost/Files";
    let pool = MySqlPool::connect(database_url).await?;

    // Simulate decrypt and download
    let file_id = 1;
    let row = sqlx::query("SELECT file_id, file_path, encrypted_key FROM Files WHERE file_id = ?")
        .bind(file_id)
        .fetch_one(&pool)
        .await?;

    let file_path: String = row.get("file_path");
    let encrypted_key: Vec<u8> = row.get("encrypted_key");

    // Placeholder for decryption logic
    println!(
        "Decrypting file from path: {} with key: {:?}...",
        file_path, encrypted_key
    );

    println!("File decrypted and downloaded successfully!");

    Ok(())
}
