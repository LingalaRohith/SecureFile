CREATE TABLE UserAccessControl (
    user_id INT,
    file_id INT,
    access_granted BOOLEAN DEFAULT FALSE,
    role_required ENUM('admin', 'developer', 'manager', 'director') NOT NULL,
    time_restricted BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (user_id, file_id),
    FOREIGN KEY (user_id) REFERENCES Users(user_id),
    FOREIGN KEY (file_id) REFERENCES Files(file_id)
);
