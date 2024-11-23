#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
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
                encrypted_key TEXT, -- Use TEXT instead of String
                priority_level TINYINT
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
    
        pool
    }

}
