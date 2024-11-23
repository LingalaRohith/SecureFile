use mockall::mock;
use std::sync::Arc;
use sqlx::{MySqlPool};
use secure_file_management::menu;


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
    use secure_file_management::file::{FileLockManager};
    // use secure_file_management::user::authenticate_user;
    
    #[tokio::test]
    async fn test_database_connection() {
        let database_url = "mysql://root:123@Rohith@localhost:3306/Files";
        let result = MySqlPool::connect(database_url).await;

        assert!(result.is_ok(), "Database connection failed");
    }


#[tokio::test]
async fn test_decrypt_and_download_file_not_found() {
    // Mock the database connection
    let pool = MySqlPool::connect("mysql://root:123%40Rohith@localhost:3306/Files").await.unwrap();

    // Call the function with a non-existing file ID
    let result = menu::decrypt_and_download_file(&pool, 9999).await;

    // Assertions
    assert!(
        result.is_err(),
        "Expected an error for non-existent file ID"
    );

    // Compare error messages precisely
    let error_message = result.err().unwrap().to_string();
    assert!(
        error_message.contains("Row not found"),
        "Unexpected error message: {}",
        error_message
    );
}
}



