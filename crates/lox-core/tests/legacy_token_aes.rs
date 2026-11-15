use lox_core::legacy_token_aes::{DecryptError, decrypt, encrypt, encrypt_command};

fn unhex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn independent_vectors_boundaries_and_opaque_bytes() {
    let key = std::array::from_fn(|i| i as u8);
    let iv = std::array::from_fn(|i| i as u8);
    // Independent Python cryptography/OpenSSL AES-CBC/PKCS7 outputs.
    let vectors = [
        (0, "e9c3ef8ab23453e6f0749cd636e7a88e"),
        (1, "cab6bf3990d9768d8e1262e8adcda55f"),
        (15, "62b5e1438a1f9d3523ac06b82b425cab"),
        (
            16,
            "f29000b62a499fd0a9f39a6add2e778053c8742d0ea29b2712f6c7af4048f4b4",
        ),
        (
            17,
            "f29000b62a499fd0a9f39a6add2e7780ae2c7b7cc6473eb497cabbd193c6174e",
        ),
        (
            31,
            "f29000b62a499fd0a9f39a6add2e7780a76daa7a1c9772567b358d49b169bc8d",
        ),
        (
            32,
            "f29000b62a499fd0a9f39a6add2e77809543b86fc046fa883a9446b82e47d12d371a2690c225b574ad74b7066f379d8d",
        ),
    ];
    for (length, expected) in vectors {
        let plaintext: Vec<u8> = (0..length).collect();
        let before = plaintext.clone();
        let expected = unhex(expected);
        assert_eq!(encrypt(&key, &iv, &plaintext), expected);
        assert_eq!(encrypt(&key, &iv, &plaintext), expected);
        assert_eq!(decrypt(&key, &iv, &expected).unwrap(), plaintext);
        assert_eq!(plaintext, before);
        assert_eq!(key, std::array::from_fn(|i| i as u8));
        assert_eq!(iv, std::array::from_fn(|i| i as u8));
    }
    let plaintext = "Grüße\0雪".as_bytes();
    let ciphertext = unhex("f004dee28fb3268301a39e80b24078aa");
    let before = ciphertext.clone();
    assert_eq!(encrypt(&key, &iv, plaintext), ciphertext);
    assert_eq!(decrypt(&key, &iv, &ciphertext).unwrap(), plaintext);
    assert_eq!(ciphertext, before);
    let binary = [0xff, 0, 0x80, 0xfe];
    assert_eq!(
        decrypt(&key, &iv, &encrypt(&key, &iv, &binary)).unwrap(),
        binary
    );
}

#[test]
fn malformed_ciphertext_is_atomic_and_errors_contain_no_input() {
    let key = [0x42; 32];
    let iv = [0x13; 16];
    let mut bad_padding = encrypt(&key, &iv, &[0; 16]);
    // Flip the last byte of the preceding block: final 0x10 padding becomes 0x11.
    bad_padding[15] ^= 1;
    for ciphertext in [vec![], vec![0], vec![0; 15], vec![0; 17], bad_padding] {
        let before = ciphertext.clone();
        let error = decrypt(&key, &iv, &ciphertext).unwrap_err();
        assert_eq!(error, DecryptError);
        assert_eq!(error.to_string(), "Unpad Error");
        assert_eq!(format!("{error:?}"), "DecryptError");
        assert!(std::error::Error::source(&error).is_none());
        assert_eq!(ciphertext, before);
    }
}

#[test]
fn deterministic_command_envelope_is_not_url_encoded() {
    let key = std::array::from_fn(|i| i as u8);
    let iv = std::array::from_fn(|i| i as u8);
    assert_eq!(
        encrypt_command(&key, &iv, ""),
        "jdev/sys/enc/6cPvirI0U+bwdJzWNueojg=="
    );
    assert_eq!(
        encrypt_command(&key, &iv, "Grüße\0雪"),
        "jdev/sys/enc/8ATe4o+zJoMBo56AskB4qg=="
    );
}
