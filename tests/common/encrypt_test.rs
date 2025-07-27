use crate::init_test;
use convertor::common::encrypt::{decrypt, encrypt};

#[test]
fn test_encrypt_and_decrypt() -> color_eyre::Result<()> {
    init_test();

    let secret = b"abcdefg"; // 密钥必须是32字节
    let message = "This is a secret message.";

    // 加密
    let encrypted = encrypt(secret, message)?;

    // 解密
    let decrypted = decrypt(secret, &encrypted)?;

    assert_eq!(message, decrypted);

    Ok(())
}
