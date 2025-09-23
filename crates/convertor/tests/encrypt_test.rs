use convertor::common::encrypt::{decrypt, encrypt};
use convertor::init_test;

#[test]
fn test_encrypt_and_decrypt() -> color_eyre::Result<()> {
    init_test!();

    let secret = b"abcdefg"; // 密钥必须是32字节
    let message = "This is a secret message.";

    // 加密
    let encrypted = encrypt(secret, message)?;
    insta::assert_snapshot!(encrypted, @"drjgraDxPZBAXWrlU4a9KL3SGbigje0acGn9tTwYzGmZBvRxAJCqF8bbX4IvnkC7n6rWzLVMOk8ndUZ85yvQXKY");

    // 解密
    let decrypted = decrypt(secret, &encrypted)?;
    insta::assert_snapshot!(decrypted, @"This is a secret message.");

    Ok(())
}

#[test]
fn test_decrypt() -> color_eyre::Result<()> {
    init_test!();

    let secret = "bppleman";

    let dec = decrypt(
        secret.as_bytes(),
        "qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3UN2hjcQ3B_IH9FI51WQX94QusyP4URwR4naCdMYFGV6aljrLzyNRhsJg9Cj55JszewkvSRXW5zMgUJCkai79FKZ4",
    )?;
    insta::assert_snapshot!(dec, @"http://127.0.0.1:64287/subscription?token=bppleman");

    let dec = decrypt(
        secret.as_bytes(),
        "qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3UN2hjcQ3B_IH9FI51WQX94QusiHxXxwR4naCdMYFGV6aljrLzyNRhsJg9Cj55Jszewk65g-J2hWsrxSAc1sHyTK1",
    )?;
    insta::assert_snapshot!(dec, @"http://127.0.0.1:65019/subscription?token=bppleman");

    Ok(())
}
