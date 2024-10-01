
## Project Overview
This project is a secure file management system built using Rust and MySQL. It provides functionalities for user registration, authentication, role-based access control, file encryption and decryption, priority management, and comprehensive logging. The system ensures that sensitive files are securely stored and accessed only by authorized users, with robust mechanisms to maintain security and integrity.

### Key Features:

- **User Management**:
  - **Registration and Authentication**: Users can register with a unique username and password, which is securely hashed using bcrypt.
  - **Role Management**: Users are assigned roles (`admin`, `developer`, `manager`, `director`) that determine their access levels.
  - **Blocking Logic**: Implements failed login attempt tracking and blocks users after a certain threshold to prevent brute-force attacks.

- **File Management**:
  - **File Upload**: Admins can upload files, which are encrypted using AES encryption before being stored.
  - **File Retrieval**: Users can retrieve and decrypt files if they have the necessary permissions based on their role and the file's priority level.
  - **Priority Management**: Each file is assigned a priority level (1-5) that dictates access restrictions.

- **Encryption and Decryption**:
  - **AES Encryption**: Utilizes AES-256 encryption for securing files.
  - **Key Management**: Generates secure encryption keys and stores them in the database with support for master keys that can expire.
  - **Secure Randomization**: Uses secure random number generation for encryption keys and initialization vectors (IVs).

- **Access Control**:
  - **Role-Based Access**: Access to files is controlled based on user roles and file priority levels.
  - **Time-Restricted Access**: Optionally restricts file access to specific time frames (e.g., office hours).
  - **Access Control Module**: The `accesscontrol.rs` module manages the logic for verifying user permissions.

- **Logging and Monitoring**:
  - **Access Logs**: Records all file access attempts, including user ID, file ID, timestamp, and success status.
  - **Failed Attempts**: Tracks failed login attempts for security monitoring.
  - **Integrity Checks**: Performs regular checks to ensure file integrity and detect unauthorized modifications.

### Modules Overview:

#### `user.rs`

- **Purpose**: Handles user registration, authentication, role management, and blocking logic.
- **Key Functions**:
  - `register`: Allows new users to register by providing a username, password, and role. Checks for username uniqueness and securely hashes the password.
  - `authenticate`: Verifies user credentials. Increments failed attempt counters and blocks the user if necessary.
  - `block_user`: Blocks a user after exceeding allowed failed login attempts.
  - `exists`: Helper function to check if a username already exists in the database.

#### `file.rs`

- **Purpose**: Manages file upload, encryption, priority management, and retrieval.
- **Key Functions**:
  - `upload`: Enables admins to upload and encrypt files, store metadata in the database, and save encrypted files to the filesystem.
  - `retrieve`: Allows users to decrypt and access files if they have the appropriate permissions and provide correct credentials.
  - `delete`: Permits admins to delete files from both the database and filesystem.

#### `encryption.rs`

- **Purpose**: Handles encryption and decryption of files using AES-256.
- **Key Functions**:
  - `generate_key`: Generates a secure 256-bit encryption key using a secure random number generator.
  - `encrypt_file`: Encrypts file data using AES-256 in CBC mode with PKCS7 padding. Generates a random IV for each encryption.
  - `decrypt_file`: Decrypts encrypted file data using the provided key and IV.

#### `accesscontrol.rs`

- **Purpose**: Manages access control logic, determining user permissions based on roles, file priority levels, and time restrictions.
- **Key Functions**:
  - `check_access`: Verifies if a user has the necessary permissions to access a specific file.
  - `is_within_allowed_time`: (Optional) Checks if the current time falls within allowed access times for time-restricted files.

### Database Schema:

1. **Users Table**:
   - Stores user credentials, roles, failed login attempts, and account status.
   - Fields: `user_id`, `username`, `password_hash`, `role`, `failed_attempts`, `is_blocked`, `created_at`.

2. **Files Table**:
   - Stores file metadata, encrypted keys, priority levels, and uploader information.
   - Fields: `file_id`, `file_name`, `file_path`, `encrypted_key`, `priority_level`, `uploaded_by`, `created_at`.

3. **AccessLogs Table**:
   - Logs each file access attempt with details for auditing and monitoring.
   - Fields: `access_id`, `file_id`, `user_id`, `access_time`, `was_successful`.

4. **UserAccessControl Table**:
   - Defines access permissions for users on specific files, including role requirements and time restrictions.
   - Fields: `user_id`, `file_id`, `access_granted`, `role_required`, `time_restricted`.

5. **MasterKeys Table**:
   - Manages master encryption keys used for file encryption, including their validity periods.
   - Fields: `key_id`, `master_key`, `valid_until`, `is_active`, `created_at`.

6. **FailedAttempts Table**:
   - Records failed login attempts for security analysis and user blocking logic.
   - Fields: `attempt_id`, `user_id`, `attempt_time`.

7. **IntegrityChecks Table**:
   - Stores results of integrity checks performed on files.
   - Fields: `check_id`, `file_id`, `check_time`, `integrity_status`.

### Cargo Dependencies:

- **Encryption and Security**:
  - `aes = "0.6"`: Provides AES encryption functionalities.
  - `bcrypt = "0.11"`: Used for hashing user passwords securely.
  - `rand`: (Implicit through other crates) Used for generating secure random numbers.

- **Database and ORM**:
  - `diesel = { version = "2.0", features = ["mysql"] }`: An ORM for interacting with the MySQL database.
  - `mysqlclient-sys = "0.4.0"`: Low-level bindings for MySQL client libraries.

- **Utilities**:
  - `chrono = "0.4"`: Handles date and time operations.
  - `dotenv = "0.15"`: Loads environment variables from a `.env` file.
  - `clap = "3.0"`: Parses command-line arguments for running the application.

- **Other Dependencies**:
  - `base64 = "0.13"`: Used for encoding/decoding encryption keys.
  - `block-modes`, `block-padding`: (Implicit through encryption crates) Used for encryption modes and padding schemes.

### Setup Instructions:

1. **Prerequisites**:
   - **Rust and Cargo**: Ensure Rust and Cargo are installed. You can install them from [rustup.rs](https://rustup.rs/).
   - **MySQL Server**: Install and run a MySQL server instance.

2. **Database Setup**:
   - Create a new database (e.g., `Files`) in MySQL.
   - Run the `schema.sql` script to set up the database schema:
     ```bash
     mysql -u username -p Files < schema.sql
     ```
   - Configure database connection settings in a `.env` file at the project root:
     ```
     DATABASE_URL=mysql://username:password@localhost/Files
     ```

3. **Build the Project**:
   - Clone the repository:
     ```bash
     git clone https://github.com/yourusername/secure_file_management.git
     ```
   - Navigate to the project directory:
     ```bash
     cd secure_file_management
     ```
   - Build the project using Cargo:
     ```bash
     cargo build
     ```


### Security Considerations:

- **Password Security**: User passwords are hashed using bcrypt to prevent plaintext storage.
- **Encryption Keys**: Encryption keys are securely generated and stored in the database. Master keys can be set to expire, adding an additional layer of security.
- **Access Control**: Robust checks are in place to ensure users can only access files they have permissions for, based on roles and priority levels.
- **Blocking Mechanism**: Users are blocked after exceeding allowed failed login attempts, mitigating brute-force attacks.
- **Integrity Checks**: Regular integrity checks help detect any unauthorized modifications to the files.

### Logging and Monitoring:

- **Access Logs**: All file access attempts are recorded, which can be audited to detect suspicious activities.
- **Failed Login Attempts**: Tracking failed login attempts helps in identifying potential security threats.
- **Audit Trails**: Combined logging mechanisms provide a comprehensive audit trail for compliance and security analysis
