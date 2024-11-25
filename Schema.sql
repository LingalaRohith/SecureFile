use Files;
CREATE TABLE Users (
    user_id INT PRIMARY KEY AUTO_INCREMENT,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('admin', 'developer', 'manager', 'director') NOT NULL,
    failed_attempts INT DEFAULT 0,
    is_blocked BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE Files (
    file_id INT PRIMARY KEY AUTO_INCREMENT,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    encrypted_key VARBINARY(255) NOT NULL,
    priority_level INT NOT NULL,  -- Example: 1 (open for all), 5 (restricted)
    uploaded_by INT,
    FOREIGN KEY (uploaded_by) REFERENCES Users(user_id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_priority_level CHECK (priority_level BETWEEN 1 AND 5)
);


CREATE TABLE UserAccessControl (
    user_id INT,
    file_id INT,
    access_granted BOOLEAN DEFAULT FALSE,
    role_required ENUM('admin', 'developer', 'manager', 'director') NOT NULL,
    time_restricted BOOLEAN DEFAULT FALSE,  -- True if access is only within office hours
    PRIMARY KEY (user_id, file_id),
    FOREIGN KEY (user_id) REFERENCES Users(user_id),
    FOREIGN KEY (file_id) REFERENCES Files(file_id)
);

use Files;
-- ALTER TABLE Users CHANGE COLUMN username email VARCHAR(255) UNIQUE NOT NULL;
-- select * from Users;



