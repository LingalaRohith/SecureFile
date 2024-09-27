CREATE TABLE Files (
    file_id INT PRIMARY KEY AUTO_INCREMENT,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    encrypted_key VARBINARY(255) NOT NULL,
    priority_level INT NOT NULL,
    uploaded_by INT,
    FOREIGN KEY (uploaded_by) REFERENCES Users(user_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_priority_level CHECK (priority_level BETWEEN 1 AND 5)
);
