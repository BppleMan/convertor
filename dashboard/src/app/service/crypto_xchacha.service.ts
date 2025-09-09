import { Injectable } from "@angular/core";
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { randomBytes } from "@noble/ciphers/utils.js";

// === base64url（无补位）工具 ===
function bytesToBase64Url(bytes: Uint8Array): string {
    // 把 bytes 映射为二进制字符串，再 btoa
    let bin = "";
    for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function base64UrlToBytes(s: string): Uint8Array {
    let b64 = s.replace(/-/g, "+").replace(/_/g, "/");
    while (b64.length % 4) b64 += "=";
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
}

// === 文本编解码 ===
const te = new TextEncoder();
const td = new TextDecoder();

@Injectable({ providedIn: "root" })
export class Crypto_xchachaService {
    private static readonly NONCE_LEN = 24;           // 24B
    private static readonly NONCE_B64URL_LEN = 32;    // 24B -> 32 chars (url-safe, no pad)

    /** Rust 同款：不足 32B 0 填充，超过截断 */
    private normalizeKey(secret: Uint8Array | string): Uint8Array {
        const src = typeof secret === "string" ? te.encode(secret) : secret;
        const out = new Uint8Array(32);
        out.set(src.subarray(0, 32), 0);
        return out;
    }

    /** encrypt：token = base64url(nonce24) + base64url(ciphertext) */
    encrypt(secret: Uint8Array | string, plaintext: string): string {
        const key = this.normalizeKey(secret);
        const nonce = randomBytes(Crypto_xchachaService.NONCE_LEN); // 浏览器安全随机
        const aead = xchacha20poly1305(key, nonce);

        const ct = aead.encrypt(te.encode(plaintext));
        return bytesToBase64Url(nonce) + bytesToBase64Url(ct);
    }

    /** decrypt：前 32 字符是 nonce 的 base64url，后半是密文 */
    decrypt(secret: Uint8Array | string, token: string): string {
        if (token.length < Crypto_xchachaService.NONCE_B64URL_LEN) {
            throw new Error("nonce 长度不合法");
        }
        const noncePart = token.slice(0, Crypto_xchachaService.NONCE_B64URL_LEN);
        const ctPart = token.slice(Crypto_xchachaService.NONCE_B64URL_LEN);

        const nonce = base64UrlToBytes(noncePart);
        if (nonce.length !== Crypto_xchachaService.NONCE_LEN) {
            throw new Error("nonce 长度不合法");
        }
        const ciphertext = base64UrlToBytes(ctPart);

        const key = this.normalizeKey(secret);
        const aead = xchacha20poly1305(key, nonce);

        try {
            const pt = aead.decrypt(ciphertext);
            return td.decode(pt);
        } catch {
            throw new Error("解密失败");
        }
    }
}
