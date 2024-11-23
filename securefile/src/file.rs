#[warn(unused_imports)]
use aes_gcm::{Aes256Gcm, Key, Nonce}; // AES-GCM encryption
use aes_gcm::aead::{KeyInit, AeadInPlace}; // Aead trait and AeadInPlace
use rand::Rng; // For generating random keys
use std::io::{self, Read, Write};
use sqlx::mysql::MySqlPool; 
use std::error::Error;

use anyhow::{Result};
use std::fs::{File, OpenOptions};

use base64::{decode};
use hex;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Manages file locks in memory to prevent concurrent file operations.
#[allow(dead_code)]
pub struct FileLockManager {
    file_locks: RwLock<HashMap<i32, Arc<Mutex<()>>>>,
}

impl FileLockManager {
        /// Creates a new instance of `FileLockManager` with an empty set of file locks.
    ///
    /// # Returns
    /// 
    /// Returns a new `FileLockManager` instance.
    pub fn new() -> Self {
        FileLockManager {
            file_locks: RwLock::new(HashMap::new()),
        }
    }

     /// Locks a file in memory.
    ///
    /// This function ensures that only one task can lock a file at a time using a `Mutex`.
    /// 
    /// # Arguments
    /// 
    /// * `file_id` - The ID of the file to be locked.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<Mutex<()>>` for the given file lock.
    // Lock a file in memory
    #[allow(dead_code)]
    async fn lock_file(&self, file_id: i32) -> Arc<Mutex<()>> {
        let mut file_locks = self.file_locks.write().await;
        file_locks
            .entry(file_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

/// Locks a file in the database by updating the `locked` status to `TRUE`.
///
/// # Arguments
///
/// * `pool` - The MySQL connection pool for the database.
/// * `file_id` - The ID of the file to be locked.
///
/// # Returns
///
/// Returns `Result<(), Box<dyn std::error::Error>>` indicating success or failure.

pub async fn lock_file_in_db(pool: &MySqlPool, file_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to lock the file in the database
    let result = sqlx::query("UPDATE Files SET locked = TRUE WHERE file_id = ? AND locked = FALSE")
        .bind(file_id)
        .execute(pool)
        .await?;

    // If no rows were updated, it means the file is already locked
    if result.rows_affected() == 0 {
        return Err("File is already locked. Please try again later.".into());
    }

    Ok(())
}

/// Unlocks a file in the database by updating the `locked` status to `FALSE`.
///
/// # Arguments
///
/// * `pool` - The MySQL connection pool for the database.
/// * `file_id` - The ID of the file to be unlocked.
///
/// # Returns
///
/// Returns `Result<(), Box<dyn std::error::Error>>` indicating success or failure.

pub async fn unlock_file_in_db(pool: &MySqlPool, file_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("UPDATE Files SET locked = FALSE WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Checks if a file is locked in the database.
///
/// # Arguments
///
/// * `pool` - The MySQL connection pool for the database.
/// * `file_id` - The ID of the file to check.
///
/// # Returns
///
/// Returns `Result<bool, Box<dyn std::error::Error>>` indicating whether the file is locked (`true`) or not (`false`).

pub async fn check_if_file_locked(pool: &MySqlPool, file_id: i32) -> Result<bool, Box<dyn std::error::Error>> {
    // Query to check if the file is locked in the database
    let row: (bool,) = sqlx::query_as("SELECT locked FROM Files WHERE file_id = ?")
        .bind(file_id)
        .fetch_one(pool)
        .await?;

    Ok(row.0)  // Return the locked status (true or false)
}

/// Encrypts a file and saves the encrypted content to a new file using AES-GCM.
///
/// AES-GCM (Advanced Encryption Standard in Galois/Counter Mode) is a mode of encryption that combines
/// the security of AES with the efficiency of Galois/Counter Mode, providing authenticated encryption with
/// both confidentiality and integrity. This function uses the AES-256 variant of AES-GCM, which requires
/// a 256-bit key for encryption and a 12-byte nonce for each encryption operation.
///
/// # Arguments
///
/// * `file_name` - The name of the file to encrypt. The encrypted file will be saved with this name and 
///   an `.enc` extension.
/// * `file_path` - The full path to the file to encrypt.
/// * `key_input` - The 32-byte encryption key as a string. This key will be used to encrypt the file.
///
/// # Returns
///
/// Returns `Result<String, Box<dyn std::error::Error>>`, containing the path of the encrypted file or an error.
///
/// # How it works:
///
/// 1. The function opens and reads the content of the file specified by `file_path`.
/// 2. A random 12-byte nonce is generated using `rand::thread_rng()` to ensure that each encryption
///    operation is unique, even if the same key is used.
/// 3. The encryption is performed using AES-GCM with the provided 32-byte key (`Aes256Gcm`), which uses 
///    AES with a 256-bit key in Galois/Counter Mode (GCM) to provide authenticated encryption.
/// 4. The encrypted content is written to a new file, where the nonce is prepended to the encrypted data 
///    to ensure it can be used for decryption later.
///
/// # Example:
/// ```rust
/// let file_name = "example.txt";
/// let file_path = "/path/to/example.txt";
/// let key_input = "32-byte-key-here-please-ensure-it-is-exactly-32-bytes"; // Replace with a real key
///
/// let encrypted_file = encrypt_and_save_file(file_name, file_path, key_input).await.unwrap();
/// println!("Encrypted file saved at: {}", encrypted_file);
/// ```
/// 
pub async fn encrypt_and_save_file(file_name: &str, file_path: &str, key_input: String) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Read the file content
    let mut file = File::open(file_path).map_err(|e| {
        eprintln!("Failed to open file: {}. Please make sure the path is correct.", e);
        e
    })?;
    
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    // Step 2: Generate a random nonce
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen(); // Generate random bytes for the nonce
    let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes

    // Step 3: Use the provided key for encryption
    let encryption_key = Key::<Aes256Gcm>::from_slice(key_input.as_bytes()); // Use the key from user input

    // Step 4: Encrypt the content
    let cipher = Aes256Gcm::new(encryption_key);
    let mut buffer = content.clone(); // Buffer for encryption

    // Encrypt the content in place using the nonce
    cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("Encryption failed");

    // Step 5: Save the encrypted file to a new location
    let encrypted_file_path = format!("{}.enc", file_name); // Append .enc to the file name
    let mut encrypted_file = File::create(&encrypted_file_path).map_err(|e| {
        eprintln!("Failed to create encrypted file: {}. Please check your permissions.", e);
        e
    })?;
    
    encrypted_file.write_all(nonce.as_slice())?; // Prepend nonce to the encrypted file
    encrypted_file.write_all(&buffer)?;

    // Return the path of the encrypted file
    Ok(encrypted_file_path)
}


/// Handles the admin interface for file management (encryption and metadata insertion).
///
/// This function runs an interactive CLI loop where the admin can choose between encrypting a file
/// or inserting file metadata into the database.
///
/// # Arguments
///
/// * `pool` - The MySQL connection pool for the database.

pub async fn admin_file_management(pool: &MySqlPool) {
    loop {
        println!("\n--- Admin File Management ---");
        println!("1. Encrypt a File");
        println!("2. Insert File Metadata into DB");
        println!("Press Enter to exit.");

        let choice = get_input("> ");
        if choice.is_empty() {
            println!("Exiting...");
            break;
        }

        match choice.as_str() {
            "1" => encrypt_file().await,
            "2" => {
                match insert_file_metadata(pool).await {
                    Ok(_) => println!("File metadata inserted successfully!"),
                    Err(e) => eprintln!("Error inserting metadata: {}", e),
                }
            }
            _ => println!("Invalid option. Try again."),
        }
    }
}

/// Encrypts a file using the provided key and path, displaying the result to the user.
///
/// # Returns
///
/// Returns `Result<String, Box<dyn std::error::Error>>` with the encrypted file path or an error.
pub async fn encrypt_file() {
    let file_name = get_input("Enter the file name to encrypt: ");
    let file_path = get_input("Enter the full path of the file: ");
    let key_input = get_input("Enter a 32-character encryption key: ");

    match encrypt_and_save_file(&file_name, &file_path, key_input).await {
        Ok(encrypted_path) => {
            println!("File encrypted successfully!");
            println!("Encrypted File Path: {}", encrypted_path);
        }
        Err(e) => eprintln!("Error encrypting file: {}", e),
    }
}

/// Inserts file metadata into the database.
///
/// # Arguments
///
/// * `pool` - The MySQL connection pool for the database.
///
/// # Returns
///
/// Returns `Result<(), Box<dyn std::error::Error>>`, indicating success or failure.

async fn insert_file_metadata(pool: &MySqlPool) -> Result<(), Box<dyn Error>> {
    let file_name = get_input("Enter the encrypted file name: ");
    let file_path = get_input("Enter the full path of the encrypted file: ");
    let encrypted_key = get_input("Enter the encrypted key: ").into_bytes();
    let priority_level: u8 = get_input("Enter the priority level (1-5): ").parse()?;

    sqlx::query("INSERT INTO Files (file_name, file_path, encrypted_key, priority_level) VALUES (?, ?, ?, ?)")
        .bind(file_name)
        .bind(file_path)
        .bind(encrypted_key)
        .bind(priority_level)
        .execute(pool)
        .await?;

    Ok(())
}

/// Decrypts an encrypted file and returns the decrypted content.
///
/// # Arguments
///
/// * `encrypted_file_path` - The path of the encrypted file.
/// * `encrypted_key_base64` - The Base64-encoded encryption key.
///
/// # Returns
///
/// Returns `Result<String, Box<dyn std::error::Error>>`, containing the path of the decrypted file or an error.

pub async fn decrypt_file(encrypted_file_path: &str, encrypted_key_base64: &[u8],_key_input: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Open the encrypted file
    let mut file = File::open(encrypted_file_path).map_err(|e| {
        eprintln!("Failed to open encrypted file: {}. Please make sure the path is correct.", e);
        e
    })?;
    
    // Step 2: Read the nonce
    let mut nonce_bytes = [0u8; 12]; // 96-bit nonce
    file.read_exact(&mut nonce_bytes)?;
    
    // Step 3: Read the encrypted content
    let mut encrypted_content = Vec::new();
    file.read_to_end(&mut encrypted_content)?;
    println!("Nonce (hex): {:?}", hex::encode(nonce_bytes));

    // Step 4: Decode the Base64 encoded encryption key
    let _decoded_key = decode(encrypted_key_base64).map_err(|e| {
        eprintln!("Failed to decode Base64 key: {}. Please check the format.", e);
        e
    })?;

    // let decoded_key = get_input("Enter a 32-character encryption key: ");

        // Step 4: Get the Base64-encoded key from the user
        let key_input = get_input("Enter the Base64-encoded encryption key: ");
    
        // Step 5: Use key directly or try decoding from Base64
    let decoded_key = if key_input.len() == 32 {
        key_input.into_bytes()  // Use directly as bytes
    } else {
        decode(&key_input).map_err(|_| "Invalid key format. Must be 32 bytes or a valid Base64 string.")?
    };

    // Step 5: Ensure the decoded key is 32 bytes (AES-256 key length)
    if decoded_key.len() != 32 {
        return Err("Invalid key length. Decryption key must be 32 bytes for AES-256.".into());
    }

    // Step 6: Use the decoded key for decryption
    let encryption_key = Key::<Aes256Gcm>::from_slice(&decoded_key); // Use the decoded key
    let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes
    println!("Encrypted content length: {}", encrypted_content.len()); // Debug: print length of encrypted content

    // Step 7: Decrypt the content
    let cipher = Aes256Gcm::new(encryption_key);
    let mut buffer = encrypted_content.clone(); // Buffer for decryption

    // Decrypt the content in place using the nonce
    match cipher.decrypt_in_place(nonce, b"", &mut buffer) {
        Ok(_) => {
            // Step 8: Save the decrypted file to a new location
            let decrypted_file_path = format!("{}.dec", encrypted_file_path); // Append .dec to the file name
            let mut decrypted_file = File::create(&decrypted_file_path).map_err(|e| {
                eprintln!("Failed to create decrypted file: {}. Please check your permissions.", e);
                e
            })?;
            
            decrypted_file.write_all(&buffer)?;
            // Return the path of the decrypted file
            Ok(decrypted_file_path)
        }
        Err(e) =>{
            eprintln!("Decryption failed: {}", e);
    Err("Decryption failed. Please check the key and try again.".into())
        }
    }
}

#[warn(unused_variables)]
pub async fn decrypt_and_edit_file(pool: &MySqlPool, file_id: i32, _file_lock_manager: Arc<FileLockManager>) -> Result<(), Box<dyn Error>> {
    // Get user input for the encryption key and new content
    // let encryption_key_input = get_input("Enter the encryption key (Base64 encoded): ");
    eprintln!("locking file with ID: {}", file_id);
    lock_file_in_db(pool, file_id).await?;
    let new_content = get_input("Enter the new content to be added: ");

    eprintln!("Unlocking file with ID: {}", file_id);
    unlock_file_in_db(pool, file_id).await?;
    
    // Fetch the file path and encryption key from the database using the file_id
    let row: (String, Vec<u8>) = sqlx::query_as(
        "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
    )
    .bind(file_id)
    .fetch_one(pool)
    .await?;
    
    let file_path = row.0;
    let encrypted_key = row.1; // This is already in a byte format

    // Decrypt the file with the provided encryption key
    let decrypted_content = decrypt_file(&file_path, &encrypted_key, Some("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR")).await?;
    println!("Decrypted content: \n{}", decrypted_content);

    // Add new content to the decrypted file
    let updated_content = format!("{}{}", decrypted_content, new_content);
    println!("Updated content: \n{}", updated_content);
    reencrypt_and_save_file(pool, file_id, &file_path, &encrypted_key, &updated_content).await?;
    // unlock_file_in_db(pool, file_id).await?;

    println!("File edited successfully.");
    Ok(())
}

pub async fn reencrypt_and_save_file(pool: &MySqlPool,file_id: i32,file_path: &str, encryption_key: &[u8], content: &str) -> Result<(), Box<dyn Error>> {
    // Generate a new nonce
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let cipher_key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(cipher_key);

    let mut buffer = content.as_bytes().to_vec();

    cipher.encrypt_in_place(nonce, b"", &mut buffer).map_err(|e| (format!("Encryption failed: {}", e)))?;

    // Step 3: Create a new file and write the encrypted data to it
    let new_file_path = format!("{}.new", file_path); // You can create a unique file name here
    let mut new_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&new_file_path)
        .map_err(|e| format!("Failed to create new file: {}", e))?;

    // Write the nonce and encrypted content
    new_file.write_all(&nonce_bytes)?;
    new_file.write_all(&(buffer.len() as u32).to_be_bytes())?;
    new_file.write_all(&buffer)?;

    // Step 4: Update the file path in the database
    sqlx::query(
        "UPDATE Files SET file_path = ? WHERE file_id = ?",
    )
    .bind(new_file_path)
    .bind(file_id)
    .execute(pool)
    .await?;

    println!("File edited successfully. The file path in the database has been updated.");

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


// p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR