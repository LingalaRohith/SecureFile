// Provides a command-line interface to interact with the system.

pub fn upload_cli() {
    let username = get_input("Enter username: ");
    let password = get_input("Enter password: ");
    let file_name = get_input("Enter file name: ");
    let file_path = get_input("Enter file path: ");
    let priority = get_input("Enter file priority: ");

    // Call the file upload function
    let user = User::authenticate(&username, &password).unwrap();
    File::upload(&file_name, &file_path, priority.parse().unwrap(), user.user_id).unwrap();
}
