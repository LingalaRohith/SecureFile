CREATE TABLE FailedAttempts (
    attempt_id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT,
    attempt_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
);
