use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use ecies::{PublicKey, SecretKey};
use ecies::{encrypt, decrypt, utils::generate_keypair};

/*
This module implements a Rust Foreign Function Interface (FFI) crypto framework to C-based libraries e.g .a staticlib, dylib, xcframework etc

Summary:
    - Generate a private key, 
    - Generate a public key from a private key, 
    - Encrypt a message using a public key, and 
    - Decrypt a message using a private key. 

This module uses the Elliptic Curve Integrated Encryption Scheme (ECIES)
*/

/*
Generates a new secret key using the libsecp256k1 library 
It returns the hexadecimal representation of the serialized secret key as a C string.
*/

#[no_mangle]
pub unsafe extern "C" fn ecies_generate_secret_key() -> *const c_char {
    let key_pair = generate_keypair();
    // Ignore public key for now
    let secret_key = key_pair.0;

    let secret_key_buffer = secret_key.serialize();
    let secret_key_hex = hex::encode(secret_key_buffer);

    let secret_key_cstring_result = CString::new(secret_key_hex);
    let secret_key_cstr = ManuallyDrop::new(secret_key_cstring_result.unwrap());

    let secret_key_ptr = secret_key_cstr.as_ptr();

    secret_key_ptr
}


/*
Generates a public key from the given secret key
It takes a secret key as a C string and returns the corresponding public key as a C string. 
Steps: 
     - Convert the secret key from a C string to a Rust string, 
     - Decode the hexadecimal representation of the secret key, and then 
     - Generate the public key from the secret key.
*/

#[no_mangle]
pub unsafe extern "C" fn ecies_public_key_from(secret_key_ptr: *const c_char) -> *const c_char {
    let secret_key_cstr = unsafe { CStr::from_ptr(secret_key_ptr) };
    let secret_key_str_result = secret_key_cstr.to_str();
    let secret_key_str = secret_key_str_result.unwrap();
    let secret_key_string = secret_key_str.to_string();
    let secret_key_buffer = hex::decode(secret_key_string).unwrap();

    let secret_key = SecretKey::parse_slice(&secret_key_buffer[..]).unwrap();

    let public_key = PublicKey::from_secret_key(&secret_key);

    let public_key_buffer = public_key.serialize_compressed();
    let public_key_hex = hex::encode(public_key_buffer);

    let public_key_cstring_result = CString::new(public_key_hex);
    // ManuallyDrop is useful when the ownership of the underlying resource is transferred to code outside of Rust
    let public_key_cstr = ManuallyDrop::new(public_key_cstring_result.unwrap());

    let public_key_ptr = public_key_cstr.as_ptr();


    public_key_ptr
}

/*
Encrypts a message using the provided public key.
It takes a public key and a message as C strings and returns the encrypted message as a base64-encoded C string. 
Steps:
     - Convert the public key from a C string to a Rust string, 
     - Decode the hexadecimal representation of the public key, 
     - Encrypt the the message using ecies encryption
*/

#[no_mangle]
pub unsafe extern "C" fn ecies_encrypt(public_key_ptr: *const c_char, message_ptr: *const c_char) -> *const c_char {
    let public_key_cstr = unsafe { CStr::from_ptr(public_key_ptr) };
    let public_key_str_result = public_key_cstr.to_str();
    let public_key_str = public_key_str_result.unwrap();
    let public_key_string = public_key_str.to_string();
    let public_key_buffer = hex::decode(public_key_string).unwrap();

    let public_key_result = PublicKey::parse_slice(&public_key_buffer[..], None);
    let public_key = public_key_result.unwrap();

    let serialized_public_key_buffer = public_key.serialize_compressed();

    let message_cstr = unsafe { CStr::from_ptr(message_ptr) };
    let message_buffer = message_cstr.to_bytes();
    
    let encrypted_result = encrypt(&serialized_public_key_buffer, message_buffer);
    let encrypted = encrypted_result.unwrap();
    let encrypted_buffer = &encrypted[..];
    let encoded = base64::encode(encrypted_buffer);

    let encrypted_message_cstring = ManuallyDrop::new(CString::new(encoded).unwrap());
    let encrypted_message_cstr = encrypted_message_cstring.as_c_str().to_str().unwrap();

    let encrypted_message_ptr = encrypted_message_cstr.as_ptr();

    encrypted_message_ptr as *const c_char
}


/*
Decrypts a message using the provided secret key.
It takes a secret key and a message as C string and returns the decrypted message as a C string. 
Steps:
     - Convert the private key and encrypted message from C strings to Rust strings 
     - Decode the hexadecimal representation of the private key, 
     - Decrypt the message using ecies decryption
*/

#[no_mangle]
pub unsafe extern "C" fn ecies_decrypt(secret_key_ptr: *const c_char, message_ptr: *const c_char) -> *const c_char {
    let secret_key_cstr = unsafe { CStr::from_ptr(secret_key_ptr) };
    let secret_key_str_result = secret_key_cstr.to_str();
    let secret_key_str = secret_key_str_result.unwrap();
    let secret_key_string = secret_key_str.to_string();
    let secret_key_buffer = hex::decode(secret_key_string).unwrap();

    let secret_key_result = SecretKey::parse_slice(&secret_key_buffer[..]);
    let secret_key = secret_key_result.unwrap();

    let serialized_secret_key_buffer = secret_key.serialize();

    let message_cstr = unsafe { CStr::from_ptr(message_ptr) };
    let message_buffer = message_cstr.to_bytes();

    let message_decode_result = base64::decode(message_buffer);
    let message_vec = message_decode_result.unwrap();
    
    let decrypted_result = decrypt(&serialized_secret_key_buffer, &message_vec[..]);
    let decrypted = decrypted_result.unwrap();

    let decrypted_message_cstring = ManuallyDrop::new(CString::new(decrypted).unwrap());
    let decrypted_message_cstr = decrypted_message_cstring.as_c_str().to_str().unwrap();

    let decrypted_message_ptr = decrypted_message_cstr.as_ptr();

    decrypted_message_ptr as *const c_char
}

/// Expose the JNI interface for android below
#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString};
    use self::jni::sys::jstring;

    #[no_mangle]
    pub unsafe extern fn Java_io_metamask_ecies_Ecies_generateSecretKey(env: JNIEnv, _: JClass) -> jstring {
        let secret_key_ptr = ecies_generate_secret_key();
        let secret_key_cstr = CStr::from_ptr(secret_key_ptr).to_str().unwrap();
        let result = env.new_string(secret_key_cstr).unwrap();
        result.into_inner()
    }

    #[no_mangle]
    pub unsafe extern fn Java_io_metamask_ecies_Ecies_derivePublicKeyFrom(env: JNIEnv, _: JClass, secret: JString) -> jstring {
        let public_key_ptr = ecies_public_key_from(env.get_string(secret).expect("Invalid private key format").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let public_key_cstr = CStr::from_ptr(public_key_ptr).to_str().unwrap();
        let result = env.new_string(public_key_cstr).unwrap();
        result.into_inner()
    }

    #[no_mangle]
    pub unsafe extern fn Java_io_metamask_ecies_Ecies_encryptMessage(env: JNIEnv, _: JClass, pubkey: JString, message: JString) -> jstring {
        let cipher_text_ptr = ecies_encrypt(env.get_string(pubkey).expect("Invalid public key format").as_ptr(), env.get_string(message).expect("Invalid message format").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let cipher_text_cstr = CStr::from_ptr(cipher_text_ptr).to_str().unwrap();
        let result = env.new_string(cipher_text_cstr).unwrap();
        result.into_inner()
    } 

    #[no_mangle]
    pub unsafe extern fn Java_io_metamask_ecies_Ecies_decryptMessage(env: JNIEnv, _: JClass, secret: JString, message: JString) -> jstring {
        let decrypted_text_ptr = ecies_decrypt(env.get_string(secret).expect("Invalid private key format").as_ptr(), env.get_string(message).expect("Invalid message format").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let decrypted_text_cstr = CStr::from_ptr(decrypted_text_ptr).to_str().unwrap();
        let output = env.new_string(decrypted_text_cstr).unwrap();
        output.into_inner()
    }                 
}