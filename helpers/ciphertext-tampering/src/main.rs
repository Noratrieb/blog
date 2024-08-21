//! ```toml
//! [dependencies]
//! chacha20 = "0.9.1"
//! ````

use chacha20::cipher::{KeyIvInit, StreamCipher};

fn main() {
    let key: chacha20::Key = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ]
    .into();
    let mut cipher = chacha20::ChaCha20::new(&key, &[0; 12].into());

    // Encrypt
    let plaintext = "ls -l /etc";
    eprintln!("plaintext:    {:?}", plaintext);
    let mut plaintext = plaintext.as_bytes().to_vec();
    eprintln!("plaintext:    {:x?}", plaintext);
    cipher.apply_keystream(&mut plaintext);
    eprintln!("ciphertext:   {:x?}", plaintext);

    // Flipping
    flip_it(&mut plaintext);
    eprintln!("ciphertext 2: {:x?}", plaintext);

    // Decrypt
    let mut cipher = chacha20::ChaCha20::new(&key, &[0; 12].into());
    cipher.apply_keystream(&mut plaintext);
    eprintln!("plaintext 2:  {:x?}", plaintext);
    eprintln!("plaintext 2:  {:x?}", String::from_utf8(plaintext).unwrap());
}

fn flip_it(ciphertext: &mut [u8]) {
    ciphertext[0] ^= 0b0001_1110;
    ciphertext[1] ^= 0b0001_1110;
    ciphertext[4] ^= 0b0001_1110;
}