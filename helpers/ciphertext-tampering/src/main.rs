use chacha20::cipher::KeyIvInit;
use chacha20::cipher::StreamCipher;
use chacha20::cipher::StreamCipherSeek;

fn main() {
    let key: chacha20::Key = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 19, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ]
    .into();
    let nonce: chacha20::Nonce = [0; 12].into();
    let mut cipher = chacha20::ChaCha20::new(&key, &nonce);

    let mut plaintext = "ls -l /etc".as_bytes().to_vec();
    eprintln!("{:x?}", plaintext);
    cipher.apply_keystream(&mut plaintext);

    eprintln!("{:x?}", plaintext);
    flippit(&mut plaintext);
    eprintln!("{:x?}", plaintext);

    cipher.seek(0);
    cipher.apply_keystream(&mut plaintext);
    eprintln!("{:x?}", plaintext);
    eprintln!("{:x?}", String::from_utf8(plaintext).unwrap());
}

fn flippit(ciphertext: &mut [u8]) {
    ciphertext[0] ^= 0b0001_1110;
    ciphertext[1] ^= 0b0001_1110;
    ciphertext[4] ^= 0b0001_1110;
}