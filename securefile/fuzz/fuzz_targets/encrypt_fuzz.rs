#![no_main]
use libfuzzer_sys::fuzz_target;
use secure_file_management::file::encrypt_and_save_file; 
use std::fs;
use std::path::Path;

const OUTPUT_DIR: &str = "/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/fuzz/corpus/encrypt_fuzz"; // Output directory for encrypted files
const MIN_FILE_SIZE: usize = 12 * 1024; // Minimum file size in bytes (12KB)

fuzz_target!(|data: &[u8]| {
    // Skip inputs smaller than 12KB
    if data.len() < MIN_FILE_SIZE {
        return;
    }

    // Use a fixed valid key for testing
    let key = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR";

    // Ensure the output directory exists
    let output_dir = Path::new(OUTPUT_DIR);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).expect("Failed to create output directory");
    }

    // Generate a unique file name based on input length and hash
    let file_name = format!("test_file_{}_{}.enc", data.len(), hex::encode(&data[0..8.min(data.len())]));
    let file_path = output_dir.join(file_name);

    // Write the fuzz data to the encryption function
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        match encrypt_and_save_file(file_path.to_str().unwrap(), output_dir.to_str().unwrap(), key.to_string()).await {
            Ok(output_path) => {
                println!("Encryption successful! File saved at: {}", output_path);
            }
            Err(e) => {
                eprintln!("Encryption failed for file {}: {:?}", file_path.display(), e);
            }
        }
    });
});
























































// #![no_main]
// use libfuzzer_sys::fuzz_target;
// use secure_file_management::file::encrypt_and_save_file; // Replace with the actual path to your function
// use std::env;
// use std::str::FromStr;

// // Fuzz target for encrypt_file
// fuzz_target!(|data: &[u8]| {
//     // Convert fuzz data to simulate user input
//     let key = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR"; // Use a fixed valid key for testing
//     let file_name = format!("test_file_{}", data.len()); // Use length of fuzz data to create unique file names
//     let file_path = "/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/src/Encrypt"; // Mock file path for fuzz testing

//     // Simulate calling the encrypt_file function
//     // Since `encrypt_file` is async, we need to use an executor
//     tokio::runtime::Runtime::new().unwrap().block_on(async {
//         let _ = encrypt_and_save_file(&file_name, file_path, key.to_string()).await.unwrap_or_else(|_| String::from("Error occurred"));
//     });
// });
// rustup override unset
