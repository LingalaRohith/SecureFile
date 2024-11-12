use aes_gcm::{Aes256Gcm, Key, Nonce}; // AES-GCM encryption
use aes_gcm::aead::{KeyInit, AeadInPlace}; // Aead trait and AeadInPlace
use rand::Rng; // For generating random keys
// use std::fs::File;
use std::io::{self, Read, Write};
use sqlx::mysql::MySqlPool; 
use std::error::Error;
// use anyhow::{Result, Context};

use aes_gcm::{ aead::{Aead, generic_array::GenericArray}};
use anyhow::{Result, Context};
use std::fs::{File, OpenOptions};
// use std::io::{self, Read, Write};
// use rand::Rng;

// use std::fs::{OpenOptions};
// use aes::{Aes256, NewBlockCipher};
// use aes::cipher::{BlockEncrypt, BlockCipher};
// use aes::block_cipher::generic_array::GenericArray;

// use aes::cipher::{BlockEncrypt, BlockCipher, KeyInit};
// use aes::Aes256;
// use aes::cipher::generic_array::GenericArray;
// use base64::{encode, decode};

use aes::{Aes256, NewBlockCipher};
// use aes::cipher::{BlockEncrypt, generic_array::GenericArray};
use base64::{encode, decode};
use hex;
// #[warn(dead_code)]
// pub struct FileInfo {
//     pub file_name: String,
//     pub file_path: String,
//     pub priority_level: u8,
//     pub encrypted_key: Vec<u8>,
//     // pub created_at: DateTime<Utc>,
// }
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::RwLock;
pub struct FileLockManager {
    file_locks: RwLock<HashMap<i32, Arc<Mutex<()>>>>,
}

impl FileLockManager {
    pub fn new() -> Self {
        FileLockManager {
            file_locks: RwLock::new(HashMap::new()),
        }
    }

    // Lock a file in memory
    async fn lock_file(&self, file_id: i32) -> Arc<Mutex<()>> {
        let mut file_locks = self.file_locks.write().await;
        file_locks
            .entry(file_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

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

pub async fn unlock_file_in_db(pool: &MySqlPool, file_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("UPDATE Files SET locked = FALSE WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn check_if_file_locked(pool: &MySqlPool, file_id: i32) -> Result<bool, Box<dyn std::error::Error>> {
    // Query to check if the file is locked in the database
    let row: (bool,) = sqlx::query_as("SELECT locked FROM Files WHERE file_id = ?")
        .bind(file_id)
        .fetch_one(pool)
        .await?;

    Ok(row.0)  // Return the locked status (true or false)
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

// pub async fn decrypt_file(encrypted_file_path: &str, key_input: String) -> Result<String, Box<dyn std::error::Error>> {
//     // Step 1: Open the encrypted file
//     let mut file = File::open(encrypted_file_path).map_err(|e| {
//         eprintln!("Failed to open encrypted file: {}. Please make sure the path is correct.", e);
//         e
//     })?;
    
//     // Step 2: Read the nonce
//     let mut nonce_bytes = [0u8; 12]; // 96-bit nonce
//     file.read_exact(&mut nonce_bytes)?;
    
//     // Step 3: Read the encrypted content
//     let mut encrypted_content = Vec::new();
//     file.read_to_end(&mut encrypted_content)?;
//     println!("Nonce (hex): {:?}", hex::encode(nonce_bytes));

//     // Step 4: Use the provided key for decryption
//     let encryption_key = Key::<Aes256Gcm>::from_slice(key_input.as_bytes()); // Use the key from user input
//     let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes
//     println!("Encrypted content length: {}", encrypted_content.len()); // Debug: print length of encrypted content


//     // Step 5: Decrypt the content
//     let cipher = Aes256Gcm::new(encryption_key);
//     let mut buffer = encrypted_content.clone(); // Buffer for decryption

//     // Decrypt the content in place using the nonce
//     match cipher.decrypt_in_place(nonce, b"", &mut buffer) {
//         Ok(_) => {
//             // Step 6: Save the decrypted file to a new location
//             let decrypted_file_path = format!("{}.dec", encrypted_file_path); // Append .dec to the file name
//             let mut decrypted_file = File::create(&decrypted_file_path).map_err(|e| {
//                 eprintln!("Failed to create decrypted file: {}. Please check your permissions.", e);
//                 e
//             })?;
            
//             decrypted_file.write_all(&buffer)?;
//             // Return the path of the decrypted file
//             Ok(decrypted_file_path)
//         }
//         Err(_) => Err("Decryption failed. Please check the key and try again.".into()),
//     }
// }

// use base64::decode;
// use aes_gcm::{Aes256Gcm, Key, Nonce}; // Ensure you have these imports

pub async fn decrypt_file(encrypted_file_path: &str, encrypted_key_base64: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
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
    let decoded_key = decode(encrypted_key_base64).map_err(|e| {
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

// use aes_gcm::{Aes256Gcm, Key, Nonce}; // Import AES-GCM
// use std::fs::File;
// use std::io::{Read, Write};
// use std::error::Error;

// pub async fn decrypt_file(encrypted_file_path: &str, encrypted_key_base64: &Vec<u8>) -> Result<String, Box<dyn Error>> {
//     // Step 1: Open the encrypted file
//     let mut file = File::open(encrypted_file_path).map_err(|e| {
//         eprintln!("Failed to open encrypted file: {}. Please make sure the path is correct.", e);
//         e
//     })?;
    
//     // Step 2: Use the provided decryption key
//     let decoded_key = encrypted_key_base64;
    
//     if decoded_key.len() != 32 {
//         return Err("Invalid key length. Decryption key must be 32 bytes for AES-256.".into());
//     }
    
//     let encryption_key = Key::<Aes256Gcm>::from_slice(&decoded_key); // Use the decoded key
//     let cipher = Aes256Gcm::new(encryption_key);

//     // Step 3: Read the nonce (12 bytes) from the file
//     let mut nonce_bytes = [0u8; 12];
//     file.read_exact(&mut nonce_bytes)?;
//     let nonce = Nonce::from_slice(&nonce_bytes);
    
//     // Step 4: Read the encrypted content length (4 bytes) from the file
//     let mut length_bytes = [0u8; 4];
//     file.read_exact(&mut length_bytes)?;
//     let encrypted_content_len = u32::from_be_bytes(length_bytes) as usize;

//     // Step 5: Read the encrypted content from the file
//     let mut encrypted_content = vec![0u8; encrypted_content_len];
//     file.read_exact(&mut encrypted_content)?;

//     // Step 6: Decrypt the content using AES-GCM
//     let mut buffer = encrypted_content.clone();
//     match cipher.decrypt_in_place(nonce, b"", &mut buffer) {
//         Ok(_) => {
//             // Step 7: Write the decrypted content to a new file
//             let decrypted_file_path = format!("{}.dec", encrypted_file_path); // Append .dec to the file name
//             let mut decrypted_file = File::create(&decrypted_file_path).map_err(|e| {
//                 eprintln!("Failed to create decrypted file: {}. Please check your permissions.", e);
//                 e
//             })?;
            
//             decrypted_file.write_all(&buffer)?;
//             println!("File decrypted successfully!");

//             // Return the path of the decrypted file
//             Ok(decrypted_file_path)
//         },
//         Err(e) => {
//             eprintln!("Decryption failed: {}", e);
//             Err("Decryption failed. Please check the key and try again.".into())
//         }
//     }
// }







// pub async fn edit_file(
//     _pool: &MySqlPool,
//     _file_id: i32,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let _file_name = get_input("Enter the name of the file to edit: ");
//     let _new_content = get_input("Enter the new content for the file: ");

//     // sqlx::query("UPDATE Files SET content = ? WHERE file_id = ? AND file_name = ?")
//     //     .bind(new_content)
//     //     .bind(file_id)
//     //     .bind(file_name)
//     //     .execute(pool)
//     //     .await?;

//     println!("File edited successfully!");
//     Ok(())
// }

// pub async fn edit_file(
//     pool: &MySqlPool,
//     file_id: i32,
// ) -> Result<(), Box<dyn Error>> {
//     // Get user input for new content
//     let file_name = get_input("Enter the name of the file to edit: ");
//     let new_content = get_input("Enter the new content for the file: ");
    

//     // Retrieve file path and encrypted_key based on file_id from the database
//     let row: (String, String) = sqlx::query_as(
//         "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
//     )
//     .bind(file_id)
//     .fetch_one(pool)
//     .await?;
//     print!("{}",row.1);

//     let file_path = row.0;
//     let encrypted_key_base64 = row.1;

//     // Decode the base64 encoded encryption key
//     let encrypted_key = decode(&encrypted_key_base64)?;

//     // Encrypt the new content
//     let encrypted_content = encrypt_content(&new_content, &encrypted_key)?;

//     // Open the file for appending
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(file_path)
//         .map_err(|e| format!("Failed to open file: {}", e))?;

//     // Write the encrypted content to the file
//     file.write_all(&encrypted_content)?;

//     println!("File edited successfully!");
//     Ok(())
// }

// // Function to encrypt content using AES-256
// fn encrypt_content(content: &str, key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
//     println!("{}", key.len());
//     if key.len() != 32//32
//     {
//         return Err("Invalid key size".into());
//     }



//     // let cipher = Aes256::new(&GenericArray::from_slice(key));
//     // let mut block = GenericArray::clone_from_slice(content.as_bytes());

//     let key = GenericArray::from_slice(key);
//     let cipher = Aes256::new(&key);

//     let mut block = GenericArray::clone_from_slice(content.as_bytes());

//     // Encrypt content
//     cipher.encrypt_block(&mut block);

//     // Return the encrypted content as a Vec<u8>
//     Ok(block.to_vec())
// }

// fn expand_key_to_32_bytes(key: &[u8]) -> Vec<u8> {
//     let mut expanded_key = Vec::with_capacity(32);
//     expanded_key.extend_from_slice(key);

//     // Pad with zeroes (or use any padding scheme you prefer) if the key is shorter than 32 bytes
//     while expanded_key.len() < 32 {
//         expanded_key.push(0);
//     }

//     expanded_key
// }

// use aes::Aes256;
// use block_modes::{BlockMode, Cbc};
// use block_modes::block_padding::Pkcs7;
// use generic_array::{GenericArray};
// use base64::{decode};
// use std::io::{self, Write};
// use std::fs::OpenOptions;
// use std::error::Error;
// use std::fs;

// pub async fn edit_file(
//     pool: &MySqlPool,
//     file_id: i32,
// ) -> Result<(), Box<dyn Error>> {
//     // Get user input for new content
//     let file_name = get_input("Enter the name of the file to edit: ");
//     let new_content = get_input("Enter the new content for the file: ");
    
//     // Retrieve file path and encrypted_key based on file_id from the database
//     let row: (String, String) = sqlx::query_as(
//         "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
//     )
//     .bind(file_id)
//     .fetch_one(pool)
//     .await?;
// println!("{}", row.0);
//     println!("{}", row.1);  // Print the encrypted key base64

//     let file_path = row.0;
//     let encrypted_key_base64 = row.1;
//     let key_input = get_input("Enter a 32-character encryption key: ");

//     // // Decode the base64 encoded encryption key
//     // let mut encrypted_key = decode(&encrypted_key_base64)?;

//     // // Expand the key to 32 bytes if needed (only if it is not 32 bytes)
//     // if encrypted_key.len() != 32 {
//     //     encrypted_key = expand_key_to_32_bytes(&encrypted_key);
//     // }

//     // // Encrypt the new content
//     // let encrypted_content = encrypt_content(&new_content, &encrypted_key)?;

//     let nonce_bytes: [u8; 12] = rand::thread_rng().gen(); // Generate random bytes for the nonce
//     let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes

//     // Step 3: Use the provided key for encryption
//     let encryption_key = Key::<Aes256Gcm>::from_slice(key_input.as_bytes()); // Use the key from user input

//     // Step 4: Encrypt the content
//     let cipher = Aes256Gcm::new(encryption_key);
//     let mut buffer = new_content.clone(); // Buffer for encryption

//     // Encrypt the content in place using the nonce
//     cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("Encryption failed");

//     // Open the file for appending
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(file_path)
//         .map_err(|e| format!("Failed to open file: {}", e))?;

//     // Write the encrypted content to the file
//     file.write_all(&encrypted_content)?;

//     println!("File edited successfully!");
//     Ok(())
// }

// pub async fn edit_file(
//     pool: &MySqlPool,
//     file_id: i32,
// ) -> Result<(), Box<dyn Error>> {
//     // Get user input for new content
//     let file_name = get_input("Enter the name of the file to edit: ");
//     let new_content = get_input("Enter the new content for the file: ");
    
//     // Retrieve file path and encrypted_key based on file_id from the database
//     let row: (String, String) = sqlx::query_as(
//         "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
//     )
//     .bind(file_id)
//     .fetch_one(pool)
//     .await?;
    
//     let file_path = row.0;
//     let encrypted_key_base64 = row.1;
//     let key_input = get_input("Enter a 32-character encryption key: ");
    
//     // Step 1: Generate a nonce (12-byte random value)
//     let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
//     let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes for AES-GCM encryption

//     // Step 2: Use the provided key for encryption (from user input)
//     let encryption_key = Key::<Aes256Gcm>::from_slice(key_input.as_bytes()); // Use the key from user input

//     // Step 3: Encrypt the content in place using AES-256-GCM
//     let mut buffer = new_content.clone().into_bytes(); // Convert content to bytes for encryption
//     let cipher = Aes256Gcm::new(encryption_key);

//     // Encrypt the content in place using the nonce
//     cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("Encryption failed");

//     // Open the file for appending (to store the encrypted content)
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(file_path)
//         .map_err(|e| format!("Failed to open file: {}", e))?;

//     // Write the nonce and the encrypted content to the file
//     file.write_all(&nonce_bytes)?; // Write the nonce at the beginning
//     file.write_all(&buffer)?; // Write the encrypted content

//     println!("File edited successfully!");
//     Ok(())
// }

// pub async fn edit_file(
//     pool: &MySqlPool,
//     file_id: i32,
// ) -> Result<(), Box<dyn Error>> {
//     // Get user input for new content
//     let file_name = get_input("Enter the name of the file to edit: ");
//     let new_content = get_input("Enter the new content for the file: ");
    
//     // Retrieve file path and encrypted_key based on file_id from the database
//     let row: (String, String) = sqlx::query_as(
//         "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
//     )
//     .bind(file_id)
//     .fetch_one(pool)
//     .await?;
    
//     let file_path = row.0;
//     let encrypted_key_base64 = row.1;
    
//     // Decode the Base64 encrypted key
//     let decoded_key = decode(encrypted_key_base64)?;
//     if decoded_key.len() != 32 {
//         return Err("Invalid key length. Key must be 32 bytes for AES-256.".into());
//     }
//     let encryption_key = Key::<Aes256Gcm>::from_slice(&decoded_key); // Use the decoded key

//     // Step 1: Generate a nonce (12-byte random value)
//     let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
//     let nonce = Nonce::from_slice(&nonce_bytes); // Use the nonce bytes for AES-GCM encryption

//     // Step 2: Encrypt the new content in place using AES-256-GCM
//     let mut buffer = new_content.clone().into_bytes(); // Convert content to bytes for encryption
//     let cipher = Aes256Gcm::new(encryption_key);

//     // Encrypt the content in place using the nonce
//     cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("Encryption failed");

//     // Open the file for appending (to store the encrypted content)
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(file_path)
//         .map_err(|e| format!("Failed to open file: {}", e))?;

//     // Write metadata (nonce and length of encrypted content) followed by the encrypted content
//     file.write_all(&nonce_bytes)?; // Write the nonce at the beginning of the section
//     file.write_all(&(buffer.len() as u32).to_be_bytes())?; // Write length of the encrypted content
//     file.write_all(&buffer)?; // Write the encrypted content

//     println!("File edited successfully!");
//     Ok(())
// }



// use aes_gcm::{Aes256Gcm, Key, Nonce};
// use aes_gcm::aead::{Aead, NewAead};
// use std::error::Error;
// use std::fs::{File, OpenOptions};
// use std::io::{Read, Write};
// use sqlx::MySqlPool;
// use std::io::{self};

// pub async fn edit_file(
//     pool: &MySqlPool,
//     file_id: i32,
// ) -> Result<(), Box<dyn Error>> {
//     // Step 1: Get encryption key from the user
//     let encrypted_key = get_input("Enter the encryption key (32 bytes in base64): ");
//     // let encrypted_key = base64::decode(encryption_key_input)?;
    
//     // Ensure the key length is 32 bytes (for AES-256)
//     if encrypted_key.len() != 32 {
//         return Err("Invalid key length. Key must be 32 bytes for AES-256.".into());
//     }
//     let encryption_key = Key::<Aes256Gcm>::from_slice(&encrypted_key); // Use the provided key

//     // Step 2: Retrieve file path and encrypted_key based on file_id from the database
//     let row: (String, Vec<u8>) = sqlx::query_as(
//         "SELECT file_path, encrypted_key FROM Files WHERE file_id = ?"
//     )
//     .bind(file_id)
//     .fetch_one(pool)
//     .await?;
    
//     let file_path = row.0;
//     let encrypted_key_from_db = row.1;
    
//     // Log the encrypted key for debugging
//     println!("Encryption key provided by user: {:?}", encryption_key);
//     println!("Encrypted key from DB: {:?}", encrypted_key_from_db);

//     // Step 3: Read the encrypted file content
//     let mut file = File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;
//     let mut encrypted_data = Vec::new();
//     file.read_to_end(&mut encrypted_data)?;

//     // Step 4: Extract nonce and encrypted content
//     let nonce = Nonce::from_slice(&encrypted_data[0..12]); // First 12 bytes are the nonce
//     let encrypted_content = &encrypted_data[12..]; // The rest is the encrypted content

//     // Log the nonce, encrypted content length, and key for debugging
//     println!("Nonce used for decryption: {:?}", nonce);
//     println!("Encrypted content length: {}", encrypted_content.len());

//     // Step 5: Decrypt the content
//     let cipher = Aes256Gcm::new(&encryption_key);
//     let mut decrypted_content = encrypted_content.to_vec(); // Copy content into a mutable buffer

//     match cipher.decrypt_in_place(nonce, b"", &mut decrypted_content) {
//         Ok(_) => {
//             println!("Decryption successful!");
//         }
//         Err(e) => {
//             eprintln!("Decryption failed: {}", e);
//             return Err("Decryption failed. Please check the encryption key.".into());
//         }
//     };

//     // Step 6: Get new content from the user
//     let new_content = get_input("Enter the new content for the file: ");
    
//     // Step 7: Modify the decrypted content
//     let mut content_str = String::from_utf8(decrypted_content)?;
//     content_str.push_str("\nModified Content: ");
//     content_str.push_str(&new_content); // Append the new content to the decrypted data
//     let modified_content = content_str.into_bytes();

//     // Step 8: Encrypt the modified content
//     let mut encrypted_buffer = modified_content.clone();
//     let mut rng = rand::thread_rng();
//     let nonce_bytes: [u8; 12] = rng.gen();
//     let nonce = Nonce::from_slice(&nonce_bytes);

//     match cipher.encrypt_in_place(nonce, b"", &mut encrypted_buffer) {
//         Ok(_) => {
//             println!("Content encrypted successfully!");
//         }
//         Err(e) => {
//             eprintln!("Encryption failed: {}", e);
//             return Err("Encryption failed.".into());
//         }
//     }

//     // Step 9: Append the new encrypted content back to the file
//     let mut file = OpenOptions::new()
//         .append(true)
//         .open(&file_path)
//         .map_err(|e| format!("Failed to open file: {}", e))?;

//     // Write metadata (nonce and length of encrypted content) followed by the encrypted content
//     file.write_all(&nonce_bytes)?; // Write the nonce at the beginning of the section
//     file.write_all(&(encrypted_buffer.len() as u32).to_be_bytes())?; // Write length of the encrypted content
//     file.write_all(&encrypted_buffer)?; // Write the encrypted content

//     println!("File edited and saved successfully!");
//     Ok(())
// }






// // Function to encrypt content using AES-256
// fn encrypt_content(content: &str, key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
//     println!("{}", key.len());
//     if key.len() != 32 {
//         return Err("Invalid key size".into());
//     }

//     let key = GenericArray::from_slice(key);
//     let cipher = Aes256::new(&key);

//     let mut block = GenericArray::clone_from_slice(content.as_bytes());

//     // Encrypt content
//     cipher.encrypt_block(&mut block);

//     // Return the encrypted content as a Vec<u8>
//     Ok(block.to_vec())
// }

// // Function to expand the key to 32 bytes (AES-256 requires 32 bytes key)
// fn expand_key_to_32_bytes(key: &[u8]) -> Vec<u8> {
//     let mut expanded_key = Vec::with_capacity(32);
//     expanded_key.extend_from_slice(key);

//     // Pad with zeroes (or use any padding scheme you prefer) if the key is shorter than 32 bytes
//     while expanded_key.len() < 32 {
//         expanded_key.push(0);
//     }

//     expanded_key
// }


pub async fn decrypt_and_edit_file(pool: &MySqlPool, file_id: i32, file_lock_manager: Arc<FileLockManager>) -> Result<(), Box<dyn Error>> {
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
    let decrypted_content = decrypt_file(&file_path, &encrypted_key).await?;
    println!("Decrypted content: \n{}", decrypted_content);

    // Add new content to the decrypted file
    let updated_content = format!("{}{}", decrypted_content, new_content);
    println!("Updated content: \n{}", updated_content);
    reencrypt_and_save_file(pool, file_id, &file_path, &encrypted_key, &updated_content).await?;
    // unlock_file_in_db(pool, file_id).await?;

    println!("File edited successfully.");
    Ok(())
}

async fn reencrypt_and_save_file(pool: &MySqlPool,file_id: i32,file_path: &str, encryption_key: &[u8], content: &str) -> Result<(), Box<dyn Error>> {
    // Generate a new nonce
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let cipher_key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(cipher_key);

    let mut buffer = content.as_bytes().to_vec();

    // Encrypt the content
    // cipher.encrypt_in_place(nonce, b"", &mut buffer).map_err(|e| {
    //     // Convert encryption error to a string and return as Box<dyn Error>
    //     Box::new(std::fmt::format(format_args!("Encryption failed: {}", e))) as Box<dyn Error>
    // })?;

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


    //     let mut file = OpenOptions::new()
    //     .write(true)
    //     .create(true)  // If file doesn't exist, create it
    //     .truncate(true) // Ensure the file is cleared before writing
    //     .open(file_path)?;
    
    // file.write_all(&nonce_bytes)?;
    // file.write_all(&(buffer.len() as u32).to_be_bytes())?;
    // file.write_all(&buffer)?;

    // println!("File successfully re-encrypted and saved.");
    
    // Ok(())
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