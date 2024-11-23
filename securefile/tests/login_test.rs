#[cfg(test)]
mod tests {
    use sqlx::{MySql, Pool};
    use bcrypt::{hash, DEFAULT_COST};
    use sqlx::{MySqlPool};
    use secure_file_management::user::{authenticate_user, UserRole}; 

    async fn setup_mock_database() -> Pool<MySql> {
        let database_url = "mysql://root:123%40Rohith@localhost:3306/Files"; // Use a test database
        let pool = MySqlPool::connect(database_url).await.unwrap();

        // Create a mock `Users` table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS Users_test (
                email VARCHAR(255) PRIMARY KEY,
                password_hash TEXT NOT NULL,
                is_blocked BOOLEAN NOT NULL,
                role VARCHAR(50) NOT NULL
            )
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("TRUNCATE TABLE Users_test").execute(&pool).await.unwrap();

        pool
    }

    async fn seed_mock_users(pool: &Pool<MySql>) {
        // Insert mock users into the database
        let hashed_password = hash("password123", DEFAULT_COST).unwrap();

        sqlx::query(
            "INSERT INTO Users_test (email, password_hash, is_blocked, role) VALUES (?, ?, ?, ?)"
        )
        .bind("test@example.com")
        .bind(&hashed_password)
        .bind(false)
        .bind("admin")
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO Users_test (email, password_hash, is_blocked, role) VALUES (?, ?, ?, ?)"
        )
        .bind("blocked_user@example.com")
        .bind(&hashed_password)
        .bind(true)
        .bind("developer")
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_authenticate_user_success() {
        let pool = setup_mock_database().await;
        seed_mock_users(&pool).await;

        let email = "test@example.com";
        let password = "password123";

        let result = authenticate_user(&pool, email, password).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), UserRole::Admin);
    }

    #[tokio::test]
    async fn test_authenticate_user_failure_wrong_password() {
        let pool = setup_mock_database().await;
        seed_mock_users(&pool).await;

        let email = "test@example.com";
        let password = "wrongpassword";

        let result = authenticate_user(&pool, email, password).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Incorrect password.".to_string());
    }

    #[tokio::test]
    async fn test_authenticate_user_failure_user_not_found() {
        let pool = setup_mock_database().await;

        let email = "nonexistent@example.com";
        let password = "password123";

        let result = authenticate_user(&pool, email, password).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User not found.".to_string());
    }

    #[tokio::test]
    async fn test_authenticate_user_failure_blocked_user() {
        let pool = setup_mock_database().await;
        seed_mock_users(&pool).await;

        let email = "blocked_user@example.com";
        let password = "password123";

        let result = authenticate_user(&pool, email, password).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User is blocked. Please contact admin.".to_string());
    }

    #[tokio::test]
    async fn test_authenticate_user_failure_invalid_role() {
        let pool = setup_mock_database().await;

        // Insert a user with an invalid role
        sqlx::query(
            "INSERT INTO Users_test (email, password_hash, is_blocked, role) VALUES (?, ?, ?, ?)"
        )
        .bind("invalid_role@example.com")
        .bind(hash("password123", DEFAULT_COST).unwrap())
        .bind(false)
        .bind("invalid_role")
        .execute(&pool)
        .await
        .unwrap();

        let email = "invalid_role@example.com";
        let password = "password123";

        let result = authenticate_user(&pool, email, password).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User not found.".to_string());
    }
}


// cargo test --test login -- --test-threads=1
