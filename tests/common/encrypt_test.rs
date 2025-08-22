use crate::init_test;
use convertor::common::encrypt::{decrypt, encrypt};

#[test]
fn test_encrypt_and_decrypt() -> color_eyre::Result<()> {
    init_test();

    let secret = b"abcdefg"; // 密钥必须是32字节
    let message = "This is a secret message.";

    // 加密
    let encrypted = encrypt(secret, message)?;
    insta::assert_snapshot!(encrypted, @"AAAAAAAAAAAAAAAA:ySjQlsiKDWUglgkbbu96L2ITbcWr6sIoKShyEQ5Nhc6sk+v2dWrC++4=");

    // 解密
    let decrypted = decrypt(secret, &encrypted)?;
    insta::assert_snapshot!(decrypted, @"This is a secret message.");

    Ok(())
}
