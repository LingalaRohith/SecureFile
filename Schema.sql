use FIles;
CREATE TABLE Users (
    user_id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(255) UNIQUE NOT NULL,
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

CREATE TABLE AccessLogs (
    access_id INT PRIMARY KEY AUTO_INCREMENT,
    file_id INT,
    user_id INT,
    access_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    was_successful BOOLEAN,
    FOREIGN KEY (file_id) REFERENCES Files(file_id),
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
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

CREATE TABLE MasterKeys (
    key_id INT PRIMARY KEY AUTO_INCREMENT,
    master_key VARBINARY(255) NOT NULL,
    valid_until TIMESTAMP,  -- Expiry time of the master key
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE FailedAttempts (
    attempt_id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT,
    attempt_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
);

CREATE TABLE IntegrityChecks (
    check_id INT PRIMARY KEY AUTO_INCREMENT,
    file_id INT,
    check_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    integrity_status BOOLEAN NOT NULL,  -- True for integrity preserved, False for compromised
    FOREIGN KEY (file_id) REFERENCES Files(file_id)
);






