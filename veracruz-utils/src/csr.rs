//! Certificate Singing Request generation
//!
//! ## Authors
//!
//! The Veracruz Development Team.
//!
//! ## Licensing and copyright notice
//!
//! See the `LICENSE.markdown` file in the Veracruz root directory for
//! information on licensing and copyright.

use std::{
    string::String,
    vec::Vec,
};

use ring::{rand::SystemRandom, signature::EcdsaKeyPair};
use ring::signature::KeyPair;

/// A struct to contain all of the information needed to generate a CSR (except
/// the signing key, of course)
pub struct CsrTemplate {
    /// The Template data to be filled with new values
    template: [u8; 302],
    /// The location of the public key in the `template` vec (start, end + 1)
    public_key_location: (usize, usize),
    /// The location of the signature in the `template` vec (start, end + 1)
    signature_location: (usize, usize),
    /// The location of the data that the signature will be generated over (start, end + 1)
    signature_range: (usize, usize),
    /// The location of the length field for the entire CSR (start, end + 1)
    overall_length_field_location: (usize, usize),
    /// The initial value of the length field for the entire CSR
    overall_length_initial_value: u16,
    /// The location of the leight field for the signature in the `template vec (start, end + 1)
    signature_length_field_location: (usize, usize),
    /// The initial value of the length field of the signature
    signature_length_initial_value: u8,
}

/// The tempalate data needed to generate a Certificate Signing Request for the
/// root enclave
pub const ROOT_ENCLAVE_CSR_TEMPLATE: CsrTemplate = CsrTemplate {
    template: [0x30, 0x82, 0x01, 0x2a, 0x30, 0x81, 0xd1, 0x02, 0x01, 0x00, 0x30, 0x6f, 0x31, 0x0b, 0x30, 0x09,
    0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02, 0x55, 0x53, 0x31, 0x0e, 0x30, 0x0c, 0x06, 0x03, 0x55,
    0x04, 0x08, 0x0c, 0x05, 0x54, 0x65, 0x78, 0x61, 0x73, 0x31, 0x0f, 0x30, 0x0d, 0x06, 0x03, 0x55,
    0x04, 0x07, 0x0c, 0x06, 0x41, 0x75, 0x73, 0x74, 0x69, 0x6e, 0x31, 0x11, 0x30, 0x0f, 0x06, 0x03,
    0x55, 0x04, 0x0a, 0x0c, 0x08, 0x56, 0x65, 0x72, 0x61, 0x63, 0x72, 0x75, 0x7a, 0x31, 0x15, 0x30,
    0x13, 0x06, 0x03, 0x55, 0x04, 0x0b, 0x0c, 0x0c, 0x52, 0x6f, 0x6f, 0x74, 0x20, 0x45, 0x6e, 0x63,
    0x6c, 0x61, 0x76, 0x65, 0x31, 0x15, 0x30, 0x13, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x0c, 0x56,
    0x65, 0x72, 0x61, 0x63, 0x72, 0x75, 0x7a, 0x52, 0x6f, 0x6f, 0x74, 0x30, 0x59, 0x30, 0x13, 0x06,
    0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03,
    0x01, 0x07, 0x03, 0x42, 0x00, 0x04, 0x38, 0x49, 0x8a, 0xf8, 0x85, 0x93, 0x87, 0x2c, 0x69, 0x56,
    0x3d, 0x66, 0x72, 0xdf, 0x08, 0x5f, 0xe9, 0x93, 0x8d, 0x9c, 0xad, 0xd9, 0x9e, 0x91, 0x52, 0xc8,
    0x50, 0x06, 0x97, 0x5f, 0x0b, 0x4b, 0x48, 0x18, 0x7a, 0x4f, 0x0b, 0x21, 0xe5, 0x46, 0x65, 0x9b,
    0x26, 0x37, 0x41, 0x03, 0x9b, 0x5a, 0x45, 0xaa, 0xc1, 0x2c, 0xf9, 0x4f, 0xa5, 0xc7, 0x35, 0xc8,
    0xe8, 0x7d, 0xdd, 0x3c, 0xc7, 0x89, 0xa0, 0x00, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce,
    0x3d, 0x04, 0x03, 0x02, 0x03, 0x48, 0x00, 0x30, 0x45, 0x02, 0x21, 0x00, 0xb5, 0x3d, 0xe3, 0x6c,
    0xcb, 0xe5, 0x2d, 0xea, 0x9b, 0x53, 0x69, 0x83, 0x51, 0x2f, 0xf8, 0x08, 0x90, 0x5a, 0x51, 0x8c,
    0xd0, 0xfa, 0x26, 0xe3, 0xdd, 0xed, 0x39, 0x16, 0x6e, 0x88, 0x79, 0x7c, 0x02, 0x20, 0x10, 0x9d,
    0x0a, 0x22, 0xcf, 0x73, 0x15, 0x59, 0x9b, 0xf4, 0x38, 0xeb, 0x10, 0x81, 0xa6, 0xe4, 0xe5, 0xbb,
    0x45, 0x2c, 0xf9, 0xd9, 0x9e, 0x30, 0xf9, 0x5c, 0x01, 0x9e, 0x7e, 0x90, 0x4e, 0xf5,],
    public_key_location: (149, 214),
    signature_location: (231, 302),
    signature_range: (4, 216),
    overall_length_field_location: (2, 4),
    overall_length_initial_value: 227,
    signature_length_field_location: (229, 230),
    signature_length_initial_value: 1,
};

pub fn generate_csr(template: &CsrTemplate, private_key: &EcdsaKeyPair) -> Result<Vec<u8>, String> {
    let public_key = private_key.public_key().as_ref().clone();
    println!("veracruz_utils::csr::generate_csr using public key:{:x?}", public_key);
    let mut constructed_csr = template.template.to_vec();
    if public_key.len() != (template.public_key_location.1 - template.public_key_location.0) {
        return Err(format!("veracruz_utils::csr::generate_csr Invalid length: public_key, wanted:{:?}, got:{:?}", template.public_key_location.1 - template.public_key_location.0, public_key.len()));
    }
    constructed_csr.splice(
        template.public_key_location.0..template.public_key_location.1,
        public_key.iter().cloned(),
    );

    let rng = SystemRandom::new();
    println!("veracruz_utils::csr::generate_csr generating signature across:{:02x?}", &constructed_csr[template.signature_range.0..template.signature_range.1]);
    let signature: Vec<u8> = private_key.sign(&rng, &constructed_csr[template.signature_range.0..template.signature_range.1]).unwrap().as_ref().to_vec();

    println!("veracruz_utils::csr::generate_csr signature:{:02x?}", signature);

    let signature_length = signature.len();
    constructed_csr.splice(
        template.signature_location.0..template.signature_location.1,
        signature,
    );

    let signature_field_length:u8 = (template.signature_length_initial_value + signature_length as u8) as u8;
    println!("Setting signature field length:{:?}", signature_field_length);
    constructed_csr[template.signature_length_field_location.0] = signature_field_length;

    let overall_length:u16 = (template.overall_length_initial_value + signature_length as u16) as u16;
    println!("Setting overall_length to:{:?}", overall_length);
    constructed_csr[template.overall_length_field_location.0] = ((overall_length & 0xff00) >> 8) as u8;
    constructed_csr[template.overall_length_field_location.0 + 1] = (overall_length & 0xff) as u8;

    println!("veracruz_utils::generate_csr generated csr:{:02x?}", constructed_csr);
    Ok(constructed_csr.clone())
}