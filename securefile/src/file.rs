// Handles file upload, encryption, priority management, and file retrieval.

pub struct File {
    pub file_id: u32,
    pub file_name: String,
    pub file_path: String,
    pub encrypted_key: Vec<u8>,
    pub priority_level: u8,
    pub uploaded_by: u32,  // user_id of the uploader
    pub created_at: chrono::NaiveDateTime,
}

impl File {
    /// Upload a new file (admin-only function)
    pub fn upload(file_name: &str, file_path: &str, priority: u8, uploader_id: u32) -> Result<(), String> {
        // 1. Check if the user is an admin
        let uploader = User::find_by_id(uploader_id)?;
        if uploader.role != UserRole::Admin {
            return Err("Only admins can upload files.".to_string());
        }

        // 2. Read and encrypt the file using AES
        let file_data = std::fs::read(file_path).unwrap();
        let encryption_key = encryption::generate_key();
        let encrypted_data = encryption::encrypt_file(&file_data, &encryption_key);

        // 3. Save the file and store encryption key in the database
        let new_file = File {
            file_id: 0,  // auto-incremented
            file_name: file_name.to_string(),
            file_path: file_path.to_string(),
            encrypted_key: encryption_key,
            priority_level: priority,
            uploaded_by: uploader.user_id,
            created_at: chrono::Utc::now().naive_utc(),
        };

        // Insert the new file into the database
        File::insert_into_db(new_file)?;

        // Save the encrypted file data to the file system
        std::fs::write(file_path, encrypted_data).unwrap();

        Ok(())
    }

    /// Retrieve and decrypt a file (based on access control and priority)
    pub fn retrieve(file_id: u32, user_id: u32, password: &str) -> Result<Vec<u8>, String> {
        // 1. Authenticate the user
        let user = User::find_by_id(user_id)?;
        User::authenticate(&user.username, password)?;

        // 2. Check if the user has access to the file based on priority and role
        let file = File::find_by_id(file_id)?;
        if !AccessControl::check_access(user_id, file.priority_level)? {
            return Err("Access denied".to_string());
        }

        // 3. Decrypt and return the file
        let encrypted_data = std::fs::read(&file.file_path).unwrap();
        let decrypted_data = encryption::decrypt_file(&encrypted_data, &file.encrypted_key);

        Ok(decrypted_data)
    }

    /// Delete a file (admin-only function)
    pub fn delete(file_id: u32, user_id: u32) -> Result<(), String> {
        // Only admins can delete files
        let user = User::find_by_id(user_id)?;
        if user.role != UserRole::Admin {
            return Err("Only admins can delete files.".to_string());
        }

        // Delete the file from the database and filesystem
        File::delete_from_db(file_id)?;
        let file = File::find_by_id(file_id)?;
        std::fs::remove_file(file.file_path).unwrap();

        Ok(())
    }
}
