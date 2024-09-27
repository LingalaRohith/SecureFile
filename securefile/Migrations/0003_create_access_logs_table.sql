CREATE TABLE AccessLogs (
    access_id INT PRIMARY KEY AUTO_INCREMENT,
    file_id INT,
    user_id INT,
    access_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    was_successful BOOLEAN,
    FOREIGN KEY (file_id) REFERENCES Files(file_id),
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
);