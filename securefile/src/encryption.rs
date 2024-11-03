use aes_gcm::{Aes256Gcm, Key, Nonce}; // AES-GCM encryption
use aes_gcm::aead::{KeyInit, AeadInPlace, Aead};
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
}

// Encrypts file data and saves it as <file_name>.enc
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

    cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("Encryption failed");

    // Step 5: Save the encrypted file as <file_name>.enc
    let encrypted_file_path = format!("{}.enc", file_name); // Append .enc to the file name
    let mut encrypted_file = File::create(&encrypted_file_path).map_err(|e| {
        eprintln!("Failed to create encrypted file: {}. Please check your permissions.", e);
        e
    })?;
    
    encrypted_file.write_all(nonce.as_slice())?; // Prepend nonce to the encrypted file
    encrypted_file.write_all(&buffer)?;

    Ok(encrypted_file_path)
}

// Decrypts an encrypted file and saves it as <file_name>.enc.dec
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

    match cipher.decrypt_in_place(nonce, b"", &mut buffer) {
        Ok(_) => {
            // Step 6: Save the decrypted file as <file_name>.enc.dec
            let decrypted_file_path = format!("{}.dec", encrypted_file_path); // Append .dec to the file name
            let mut decrypted_file = File::create(&decrypted_file_path).map_err(|e| {
                eprintln!("Failed to create decrypted file: {}. Please check your permissions.", e);
                e
            })?;
            
            decrypted_file.write_all(&buffer)?;
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
