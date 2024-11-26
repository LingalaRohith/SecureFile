#![no_main]

use libfuzzer_sys::fuzz_target;
use secure_file_management::file::decrypt_file;
use std::fs;
use std::path::Path;

const HARDCODED_KEY: &str = "p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR";

fuzz_target!(|data: &[u8]| {
    let key_bytes = HARDCODED_KEY.as_bytes();
    // let corpus_dir = Path::new("fuzz/corpus/encrypt_fuzz");
    let corpus_dir = Path::new("/Users/rohithlingala/Documents/Academics/Secure_Programming/SecureFile/securefile/fuzz/corpus/encrypt_fuzz");


    if let Ok(entries) = fs::read_dir(corpus_dir) {
        for entry in entries {
            if let Ok(file) = entry {
                let file_path = file.path();

                if file_path.is_file() {
                    // Skip files that are too small
                    if file_path.metadata().unwrap().len() < 13 {
                        eprintln!(
                            "Skipping file {}: Too small to be a valid encrypted file",
                            file_path.display()
                        );
                        continue;
                    }

                    // Attempt decryption
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        match decrypt_file(file_path.to_str().unwrap(), key_bytes, Some("p1QmZaT5Lk9XBNr2CjY4MWKx8Lt7FVcR")).await {
                            Ok(decrypted_path) => {
                                println!(
                                    "Decryption successful! Decrypted file saved at: {}",
                                    decrypted_path
                                );
                            }
                            Err(e) => {
                                eprintln!(
                                    "Decryption failed for file {}: {:?}",
                                    file_path.display(),
                                    e
                                );
                            }
                        }
                    });
                }
            }
        }
    } else {
        eprintln!("Failed to read corpus directory: {}", corpus_dir.display());
    }
});
