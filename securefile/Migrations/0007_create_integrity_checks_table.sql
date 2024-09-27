CREATE TABLE IntegrityChecks (
    check_id INT PRIMARY KEY AUTO_INCREMENT,
    file_id INT,
    check_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    integrity_status BOOLEAN NOT NULL,
    FOREIGN KEY (file_id) REFERENCES Files(file_id)
);
