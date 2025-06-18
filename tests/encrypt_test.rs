use convertor::encrypt::{decrypt, encrypt};

#[test]
fn test_encrypt_decrypt() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let secret = b"abcdefg"; // 密钥必须是32字节
    let message = "This is a secret message.";

    // 加密
    let encrypted = encrypt(secret, message)?;
    println!("Encrypted: {}", encrypted);

    // 解密
    let decrypted = decrypt(secret, &encrypted)?;
    println!("Decrypted: {}", decrypted);

    assert_eq!(message, decrypted);

    Ok(())
}
