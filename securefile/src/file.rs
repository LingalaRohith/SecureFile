

use aes_gcm::{Aes256Gcm, Key, Nonce}; // AES-GCM encryption
use aes_gcm::aead::{KeyInit, AeadInPlace}; // Aead trait and AeadInPlace
use rand::Rng; // For generating random keys
use std::fs::File;
use std::io::{self, Read, Write};
use sqlx::mysql::MySqlPool; 
use std::error::Error;

#[allow(dead_code)]
pub struct FileInfo {
    pub file_name: String,
    pub file_path: String,
    pub priority_level: u8,
    pub encrypted_key: Vec<u8>,
    // pub created_at: DateTime<Utc>,
}



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

async fn encrypt_file() {
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

pub async fn decrypt_file(encrypted_file_path: &str, key_input: String) -> Result<String, Box<dyn std::error::Error>> {
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

    // Step 4: Use the provided key for decryption
    let encryption_key = Key::<Aes256Gcm>::from_slice(key_input.as_bytes()); // Use the key from user input
    let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes

    // Step 5: Decrypt the content
    let cipher = Aes256Gcm::new(encryption_key);
    let mut buffer = encrypted_content.clone(); // Buffer for decryption

    // Decrypt the content in place using the nonce
    match cipher.decrypt_in_place(nonce, b"", &mut buffer) {
        Ok(_) => {
            // Step 6: Save the decrypted file to a new location
            let decrypted_file_path = format!("{}.dec", encrypted_file_path); // Append .dec to the file name
            let mut decrypted_file = File::create(&decrypted_file_path).map_err(|e| {
                eprintln!("Failed to create decrypted file: {}. Please check your permissions.", e);
                e
            })?;
            
            decrypted_file.write_all(&buffer)?;
            // Return the path of the decrypted file
            Ok(decrypted_file_path)
        }
        Err(_) => Err("Decryption failed. Please check the key and try again.".into()),
    }
}


// Helper function to get user input
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
