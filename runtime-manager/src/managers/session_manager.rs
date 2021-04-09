//! Interfaces with the session manager.
//!
//! ## Authors
//!
//! The Veracruz Development Team.
//!
//! ## Licensing and copyright notice
//!
//! See the `LICENSE.markdown` file in the Veracruz root directory for
//! information on licensing and copyright.

use crate::managers::*;
use ::session_manager::SessionContext;
use std::{sync::atomic::Ordering, vec::Vec};
use ring::{rand::SystemRandom};
use rustls::PrivateKey;

pub fn init_session_manager(policy_json: &str) -> Result<(), RuntimeManagerError> {
    let policy_hash = ring::digest::digest(&ring::digest::SHA256, &policy_json.as_bytes());
    let policy = veracruz_utils::VeracruzPolicy::from_json(policy_json)?;

    if *policy.debug() {
        super::DEBUG_FLAG.store(true, Ordering::SeqCst);
    }

    {
        let state = ProtocolState::new(policy.clone(), hex::encode(policy_hash.as_ref()))?;
        let mut protocol_state = super::PROTOCOL_STATE.lock()?;
        *protocol_state = Some(state);
    }

    //TODO: change the error type
    let new_session_manager = SessionContext::new(policy)?;

    {
        let mut session_manager_state = super::MY_SESSION_MANAGER.lock()?;
        *session_manager_state = Some(new_session_manager);
    }

    Ok(())
}

pub fn new_session() -> Result<u32, RuntimeManagerError> {
    let local_session_id = {
        let mut session_counter = super::SESSION_COUNTER.lock()?;
        *session_counter += 1;
        session_counter.clone()
    };

    let session = match &*super::MY_SESSION_MANAGER.lock()? {
        Some(my_session_manager) => my_session_manager.create_session(),
        None => {
            return Err(RuntimeManagerError::UninitializedSessionError(
                "new_session",
            ))
        }
    };

    super::SESSIONS.lock()?.insert(local_session_id, session);
    Ok(local_session_id)
}

pub fn close_session(session_id: u32) -> Result<(), RuntimeManagerError> {
    super::SESSIONS.lock()?.remove(&session_id);
    Ok(())
}

pub fn send_data(session_id: u32, input_data: &[u8]) -> Result<(), RuntimeManagerError> {
    let mut sessions = super::SESSIONS.lock()?;
    let this_session =
        sessions
            .get_mut(&session_id)
            .ok_or(RuntimeManagerError::UnavailableSessionError(
                session_id as u64,
            ))?;
    let _result = this_session.send_tls_data(&mut input_data.to_vec())?;

    //TODO: change the error type
    let plaintext_option = this_session.read_plaintext_data()?;

    let proc_ret: super::ProvisioningResponse = match plaintext_option {
        Some((client_id, roles, plaintext_data)) => {
            super::execution_engine_manager::dispatch_on_incoming_data(
                session_id,
                client_id as u64,
                &roles,
                &plaintext_data,
            )?
        }
        None => super::ProvisioningResponse::WaitForMoreData,
    };

    match proc_ret {
        super::ProvisioningResponse::ProtocolError { response } => {
            Ok(this_session.return_data(&response)?)
        }
        super::ProvisioningResponse::WaitForMoreData => Ok(()),
        super::ProvisioningResponse::Success { response } => {
            Ok(this_session.return_data(&response)?)
        }
    }
}

// TODO: The 'match' inside the and_then closure is difficult to parse
pub fn get_data(session_id: u32) -> Result<(bool, Vec<u8>), RuntimeManagerError> {
    let (result, _) = match super::SESSIONS.lock()?.get_mut(&session_id) {
        Some(this_session) => {
            let result = this_session.read_tls_data();
            let needed = this_session.read_tls_needed();
            //TODO: change the error type
            Ok((result, needed))
        }
        None => Err(RuntimeManagerError::UnavailableSessionError(
            session_id as u64,
        )),
    }?;

    let active_flag = super::PROTOCOL_STATE.lock()?.is_some();

    match result? {
        Some(output_data) => Ok((active_flag, output_data)),
        None => Err(RuntimeManagerError::NoDataError),
    }
}

// TODO: the 'match' inside the 'and_then' closure is difficult to parse
pub fn get_data_needed(session_id: u32) -> Result<bool, RuntimeManagerError> {
    match super::SESSIONS.lock()?.get_mut(&session_id) {
        Some(this_session) => Ok(this_session.read_tls_needed()),
        None => Err(RuntimeManagerError::UnavailableSessionError(
            session_id as u64,
        )),
    }
}

pub fn get_enclave_cert_pem() -> Result<Vec<u8>, RuntimeManagerError> {
    match &*super::MY_SESSION_MANAGER.lock()? {
        Some(my_session_manager) => Ok(my_session_manager.server_certificate_buffer().clone()),
        None => Err(RuntimeManagerError::UninitializedSessionError(
            "get_enclave_cert_pem",
        )),
    }
}

pub fn get_enclave_cert() -> Result<rustls::Certificate, RuntimeManagerError> {
    match &*super::MY_SESSION_MANAGER.lock()? {
        Some(my_session_manager) => Ok(my_session_manager.server_certificate().clone()),
        None => Err(RuntimeManagerError::UninitializedSessionError(
            "get_enclave_cert",
        )),
    }
}

fn get_enclave_public_key() -> Result<Vec<u8>, RuntimeManagerError> {
    match &*super::MY_SESSION_MANAGER.lock()? {
        Some(session_manager) => {
            return Ok(session_manager.public_key());
        },
        None => {
            return Err(RuntimeManagerError::UninitializedSessionError("get_enclave_public_key"));
        },
    }
}

fn get_enclave_private_key() -> Result<PrivateKey, RuntimeManagerError> {
    match &*super::MY_SESSION_MANAGER.lock()? {
        Some(session_manager) => {
            return Ok(session_manager.private_key());
        },
        None => {
            return Err(RuntimeManagerError::UninitializedSessionError("get_enclave_private_key"));
        },
    }
}

pub fn generate_csr() -> Result<Vec<u8>, RuntimeManagerError> {
    let private_key_vec = get_enclave_private_key()?.0;
    let private_key = ring::signature::EcdsaKeyPair::from_pkcs8(&ring::signature::ECDSA_P256_SHA256_ASN1_SIGNING, &private_key_vec)
        .map_err(|err| RuntimeManagerError::RingKeyRejected(err))?;
    println!("runtime-manager::session_manager::generate_csr calling veracruz_utils::csr::generate_csr");
    let csr = veracruz_utils::csr::generate_csr(&private_key).unwrap();
    return Ok(csr);
}

pub fn get_enclave_name() -> Result<std::string::String, RuntimeManagerError> {
    match &*super::MY_SESSION_MANAGER.lock()? {
        Some(my_session_manager) => Ok(my_session_manager.name().clone()),
        None => Err(RuntimeManagerError::UninitializedSessionError(
            "get_enclave_name",
        )),
    }
}
