use rsa::{PaddingScheme, PublicKey, RsaPublicKey};

pub fn verify(public_key: RsaPublicKey, message: &[u8], signature: &[u8]) -> bool {
    let padding = PaddingScheme::new_pss_with_salt::<md5::Md5>(48usize);
    public_key.verify(padding, message, signature).is_ok()
}
