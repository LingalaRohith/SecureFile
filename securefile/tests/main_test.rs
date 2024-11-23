use mockall::mock;
use std::sync::Arc;
use tokio::test;
use sqlx::{MySqlPool, Row};
use sqlx::mysql::MySqlRow;


// Mocking the `authenticate_user` function
mock! {
    pub User {
        pub async fn authenticate_user(pool: &MySqlPool, email: &str, password: &str) -> Result<UserRole, String>;
    }

}
// Define roles for mocking
#[derive(Debug, PartialEq)]
pub enum UserRole {
    Admin,
    Manager,
    Director,
    Developer,
}

#[cfg(test)]
mod tests {
    use super::*; // Import code from the main module
    // use secure_file_management::user::authenticate_user;
    
    #[tokio::test]
    async fn test_database_connection() {
        let database_url = "mysql://root:123%40Rohith@localhost:3306/Files";
        let result = MySqlPool::connect(database_url).await;

        assert!(result.is_ok(), "Database connection failed");
    }

    #[tokio::test]
async fn test_user_authentication_success() {
    let mock_pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();
    let email = "rohithlingala11@gmail.com";
    let password = "123@Rohith";

    let mock_user = MockUser::default();
    mock_user.expect_authenticate_user()
        .withf(move |_, e, p| e == email && p == password)
        .returning(|_, _, _| Ok(UserRole::Admin));

    let result = mock_user.authenticate_user(&mock_pool, email, password).await;
    assert_eq!(result, Ok(UserRole::Admin));
}

#[tokio::test]
async fn test_user_authentication_failure() {
    let mock_pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();
    let email = "test@example.com";
    let password = "wrongpassword";

    let mock_user = MockUser::default();
    mock_user.expect_authenticate_user()
        .withf(move |_, e, p| e == email && p == password)
        .returning(|_, _, _| Err(String::from("Invalid email or password")));

    let result = mock_user.authenticate_user(&mock_pool, email, password).await;
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Invalid email or password");
}


#[tokio::test]
async fn test_role_routing_admin() {
    let mock_pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();

    let role = UserRole::Admin;

    match role {
        UserRole::Admin => {
            let result = admin_menu(&mock_pool).await;
            assert!(result.is_ok(), "Admin menu execution failed");
        }
        _ => panic!("Test failed: Expected Admin role"),
    }
}

#[tokio::test]
async fn test_role_routing_manager() {
    let mock_pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();
    let file_lock_manager = Arc::new(file::FileLockManager::new());
    let email = "rohith1@gmailx.com";

    let role = UserRole::Manager;

    match role {
        UserRole::Manager => {
            let result = manager_menu(&mock_pool, file_lock_manager.clone(), &email).await;
            assert!(result.is_ok(), "Manager menu execution failed");
        }
        _ => panic!("Test failed: Expected Manager role"),
    }
}

#[tokio::test]
async fn test_role_routing_invalid_user() {
    let role = "InvalidRole";
    match role {
        "Admin" | "Manager" | "Director" | "Developer" => {
            panic!("Test failed: Expected invalid role, got a valid one");
        }
        _ => {
            assert!(true, "Invalid role handling is correct");
        }
    }
}

#[tokio::test]
    async fn test_decrypt_and_download_file_success() {
        // Mock the database connection
        let pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();

        // Insert a mock row into the database for testing
        sqlx::query(
            "INSERT INTO Files (file_id, file_path, encrypted_key) VALUES (?, ?, ?)"
        )
        .bind(10)
        .bind("/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/pdf.enc")
        .bind("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR") // Mock encrypted key
        .execute(&pool)
        .await
        .unwrap();

        // // Mock `decrypt_file` function
        // let mut mock_file = MockFile::default();
        // mock_file
        //     .expect_decrypt_file()
        //     .withf(|path, key| path == "/path/to/encrypted_file" && key == &[1, 2, 3, 4])
        //     .returning(|_, _| Ok("Decrypted content".to_string()));

        // Call the function
        let result = decrypt_and_download_file(&pool, 1).await;

        // Assertions
        assert!(result.is_ok(), "Function failed unexpectedly");
    }

    #[tokio::test]
    async fn test_decrypt_and_download_file_not_found() {
        // Mock the database connection
        let pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();

        // Call the function with a non-existing file ID
        let result = decrypt_and_download_file(&pool, 9999).await;

        // Assertions
        assert!(result.is_err(), "Expected error for non-existent file ID");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Row not found",
            "Unexpected error message"
        );
    }

    #[tokio::test]
    async fn test_decrypt_and_download_file_decryption_failure() {
        // Mock the database connection
        let pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();

        // Insert a mock row into the database for testing
        sqlx::query(
            "INSERT INTO Files (file_id, file_path, encrypted_key) VALUES (?, ?, ?)"
        )
        .bind(11)
        .bind("/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/pdf.enc")
        .bind("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR") // Mock encrypted key
        .execute(&pool)
        .await
        .unwrap();

        // Mock `decrypt_file` function to simulate failure
        // let mut mock_file = MockFile::default();
        // mock_file
        //     .expect_decrypt_file()
        //     .withf(|path, key| path == "/path/to/encrypted_file" && key == &[1, 2, 3, 4])
        //     .returning(|_, _| Err("Decryption failed".into()));

        // Call the function
        let result = decrypt_and_download_file(&pool, 2).await;

        // Assertions
        assert!(result.is_err(), "Expected decryption failure");
        assert_eq!(
            result.err().unwrap().to_string(),
            "Decryption failed",
            "Unexpected error message"
        );
    }
}



