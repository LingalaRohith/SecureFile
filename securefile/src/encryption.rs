// Handles encryption and decryption of files using AES.

pub fn generate_key() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let key: [u8; 32] = rng.gen(); // Generate a random 256-bit key
    key.to_vec()
}

pub fn encrypt_file(file_data: &[u8], key: &[u8]) -> Vec<u8> {
    use aes::Aes256;
    use block_modes::{BlockMode, Cbc};
    use block_modes::block_padding::Pkcs7;
    use rand::Rng;

    let iv: [u8; 16] = rand::thread_rng().gen();
    let cipher = Cbc::<Aes256, Pkcs7>::new_var(key, &iv).unwrap();
    let mut buffer = file_data.to_vec();
    cipher.encrypt_vec(&buffer)
}

pub fn decrypt_file(encrypted_data: &[u8], key: &[u8]) -> Vec<u8> {
    use aes::Aes256;
    use block_modes::{BlockMode, Cbc};
    use block_modes::block_padding::Pkcs7;

    let iv: [u8; 16] = encrypted_data[0..16].try_into().unwrap(); // Extract IV
    let cipher = Cbc::<Aes256, Pkcs7>::new_var(key, &iv).unwrap();
    let mut buffer = encrypted_data[16..].to_vec();
    cipher.decrypt_vec(&mut buffer).unwrap()
}
