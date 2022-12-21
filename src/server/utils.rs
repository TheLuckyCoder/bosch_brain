use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use std::str;

pub fn parse_port(buffer: &[u8]) -> Option<u16> {
    str::from_utf8(buffer).ok()?.parse::<u16>().ok()
}

pub fn verify_signature(public_key: RsaPublicKey, message: &[u8], signature: &[u8]) -> bool {
    let padding = PaddingScheme::new_pss_with_salt::<md5::Md5>(48usize);
    public_key.verify(padding, message, signature).is_ok()
}
