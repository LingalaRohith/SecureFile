#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use secure_file_management::file::{encrypt_and_save_file, decrypt_file, reencrypt_and_save_file, decrypt_and_edit_file};
    #[tokio::test]
    async fn test_encrypt_and_save_file_success() {
        // Create a temporary directory and file for testing
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_name = "test.txt";
        let key = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR".to_string();

        // Write sample content to the file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is a test file").unwrap();

        // Call the function
        let result = encrypt_and_save_file(file_name, file_path.to_str().unwrap(), key).await;

        // Assert that encryption was successful
        assert!(result.is_ok(), "Encryption failed unexpectedly");
        let encrypted_path = result.unwrap();
        assert!(std::path::Path::new(&encrypted_path).exists(), "Encrypted file not created");
    }

    #[tokio::test]
    async fn test_decrypt_file_success() {
        // Create a temporary directory and file for testing
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.enc");
        let key = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR".to_string();

        // Simulate an encrypted file (write nonce + encrypted content)
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&[0u8; 12]).unwrap(); // Nonce
        file.write_all(&[1, 2, 3, 4]).unwrap(); // Mock encrypted data

        // Call the function
        let result = decrypt_file(file_path.to_str().unwrap(), key.as_bytes()).await;

        // Assert that decryption was successful
        assert!(result.is_ok(), "Decryption failed unexpectedly");
        let decrypted_content = result.unwrap();
        assert_eq!(decrypted_content, "This is a test file", "Decrypted content mismatch");
    }


    #[tokio::test]
    async fn test_decrypt_and_edit_file_success() {
        let pool = setup_mock_database().await;
        let file_id = 1;

        // Insert mock data into the database
        sqlx::query("INSERT INTO Files (file_id, file_path, encrypted_key) VALUES (?, ?, ?)")
            .bind(file_id)
            .bind("/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/renke.enc")
            .bind("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR")
            .execute(&pool)
            .await
            .unwrap();

        // Mock the decryption function
        let mock_decrypted_content = "Original content";
        let mock_input = "Appended content";

        // Mock user input for new content
        let result = decrypt_and_edit_file(&pool, file_id, std::sync::Arc::new(secure_file_management::file::FileLockManager::new())).await;

        // Assert that the operation was successful
        assert!(result.is_ok(), "Decrypt and edit failed unexpectedly");
    }


    #[tokio::test]
    async fn test_reencrypt_and_save_file_success() {
        let pool = setup_mock_database().await;
        let file_path = "/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/renke.enc";
        let encryption_key = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR".as_bytes();
        let content = "Reencrypted content";

        // Call the function
        let result = reencrypt_and_save_file(&pool, 1, file_path, &encryption_key, content).await;

        // Assert that re-encryption was successful
        assert!(result.is_ok(), "Re-encryption failed unexpectedly");
    }

    async fn setup_mock_database() -> sqlx::MySqlPool {
        let database_url = "mysql://root:123%40Rohith@localhost:3306/Files";
        let pool = sqlx::MySqlPool::connect(database_url).await.unwrap();
    
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS Files_test (
                file_id INT PRIMARY KEY,
                file_path TEXT,
                encrypted_key String,
                priority_level TINYINT
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
    
        pool
    }

}
