// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under both the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree and the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree.

//! This module contains all the type conversions between internal AKD & message types
//! with the protobuf types

use std::convert::{TryFrom, TryInto};

use protobuf::RepeatedField;

pub mod inter_node;

// Protobuf best practice says everything should be `optional` to ensure
// maximum back-compatibility. This helper function ensures an optional
// field is present in a particular interface version.
macro_rules! require {
    ($obj:ident, $has_field:ident) => {
        if !$obj.$has_field() {
            return Err(crate::comms::CommunicationError::Serialization(format!(
                "Condition {}.{}() failed.",
                stringify!($obj),
                stringify!($has_field)
            )));
        }
    };
}

macro_rules! hash_to_bytes {
    ($obj:expr) => {
        akd::serialization::from_digest::<H>($obj).map_err(|_| {
            crate::comms::CommunicationError::Serialization(
                "Failed to convert digest to bytes".to_string(),
            )
        })?
    };
}

macro_rules! hash_from_bytes {
    ($obj:expr) => {
        akd::serialization::to_digest::<H>($obj).map_err(|_| {
            crate::comms::CommunicationError::Serialization(
                "Failed to convert bytes to digest".to_string(),
            )
        })?
    };
}

// ==============================================================
// NodeLabel
// ==============================================================

impl TryFrom<akd::node_state::NodeLabel> for inter_node::NodeLabel {
    type Error = crate::comms::CommunicationError;

    fn try_from(input: akd::node_state::NodeLabel) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        result.set_len(input.len);
        result.set_val(input.val);
        Ok(result)
    }
}

impl TryFrom<&inter_node::NodeLabel> for akd::node_state::NodeLabel {
    type Error = crate::comms::CommunicationError;

    fn try_from(input: &inter_node::NodeLabel) -> Result<Self, Self::Error> {
        require!(input, has_len);
        require!(input, has_val);
        Ok(akd::node_state::NodeLabel {
            len: input.get_len(),
            val: input.get_val(),
        })
    }
}

// ==============================================================
// Node
// ==============================================================

impl<H> TryFrom<akd::node_state::Node<H>> for inter_node::Node
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: akd::node_state::Node<H>) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        result.set_label(input.label.try_into()?);
        result.set_hash(hash_to_bytes!(input.hash));
        Ok(result)
    }
}

impl<H> TryFrom<&inter_node::Node> for akd::node_state::Node<H>
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: &inter_node::Node) -> Result<Self, Self::Error> {
        require!(input, has_label);
        require!(input, has_hash);
        let label: akd::node_state::NodeLabel = input.get_label().try_into()?;
        Ok(akd::node_state::Node::<H> {
            label,
            hash: hash_from_bytes!(input.get_hash()),
        })
    }
}

// ==============================================================
// Append-only proof
// ==============================================================

impl<H> TryFrom<akd::proof_structs::AppendOnlyProof<H>> for inter_node::AppendOnlyProof
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: akd::proof_structs::AppendOnlyProof<H>) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        let mut inserted = vec![];
        let mut unchanged = vec![];

        for item in input.inserted.into_iter() {
            inserted.push(item.try_into()?);
        }
        for item in input.unchanged_nodes.into_iter() {
            unchanged.push(item.try_into()?);
        }

        result.set_inserted(RepeatedField::from_vec(inserted));
        result.set_unchanged(RepeatedField::from_vec(unchanged));
        Ok(result)
    }
}

impl<H> TryFrom<&inter_node::AppendOnlyProof> for akd::proof_structs::AppendOnlyProof<H>
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: &inter_node::AppendOnlyProof) -> Result<Self, Self::Error> {
        let mut inserted = vec![];
        let mut unchanged = vec![];
        for item in input.get_inserted() {
            inserted.push(item.try_into()?);
        }
        for item in input.get_unchanged() {
            unchanged.push(item.try_into()?);
        }
        Ok(akd::proof_structs::AppendOnlyProof {
            inserted,
            unchanged_nodes: unchanged,
        })
    }
}

// ==============================================================
// Verify Request
// ==============================================================

impl<H> TryFrom<crate::node::messages::inter_node::VerifyRequest<H>> for inter_node::VerifyRequest
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(
        input: crate::node::messages::inter_node::VerifyRequest<H>,
    ) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        result.set_epoch(input.epoch);
        result.set_new_hash(hash_to_bytes!(input.new_hash));
        result.set_previous_hash(hash_to_bytes!(input.previous_hash));
        result.set_proof(input.append_only_proof.try_into()?);
        Ok(result)
    }
}

impl<H> TryFrom<&inter_node::VerifyRequest> for crate::node::messages::inter_node::VerifyRequest<H>
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: &inter_node::VerifyRequest) -> Result<Self, Self::Error> {
        require!(input, has_epoch);
        require!(input, has_new_hash);
        require!(input, has_previous_hash);
        let proof: akd::proof_structs::AppendOnlyProof<H> = input.get_proof().try_into()?;

        Ok(crate::node::messages::inter_node::VerifyRequest::<H> {
            epoch: input.get_epoch(),
            new_hash: hash_from_bytes!(input.get_new_hash()),
            previous_hash: hash_from_bytes!(input.get_previous_hash()),
            append_only_proof: proof,
        })
    }
}

// ==============================================================
// Verify Response
// ==============================================================

impl<H> TryFrom<crate::node::messages::inter_node::VerifyResponse<H>> for inter_node::VerifyResponse
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(
        input: crate::node::messages::inter_node::VerifyResponse<H>,
    ) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        match (input.encrypted_quorum_key_shard, input.verified_hash) {
            (Some(shard), Some(hash)) => {
                result.set_verified_hash(hash_to_bytes!(hash));
                result.set_encrypted_quorum_key_shard(shard);
            }
            _ => {
                // >= 1 of the components is missing, this is assumed a "validation failure" scenario
                // i.e. the proof failed to verify
            }
        }
        Ok(result)
    }
}

impl<H> TryFrom<&inter_node::VerifyResponse>
    for crate::node::messages::inter_node::VerifyResponse<H>
where
    H: winter_crypto::Hasher,
{
    type Error = crate::comms::CommunicationError;

    fn try_from(input: &inter_node::VerifyResponse) -> Result<Self, Self::Error> {
        if input.has_verified_hash() && input.has_encrypted_quorum_key_shard() {
            // verification succeeded on the worker node, proceed with reconstructing the result
            Ok(Self {
                verified_hash: Some(hash_from_bytes!(input.get_verified_hash())),
                encrypted_quorum_key_shard: Some(input.get_encrypted_quorum_key_shard().to_vec()),
            })
        } else {
            // Verification failed or a partial result came back. Both are mapped to verification failed
            Ok(Self {
                verified_hash: None,
                encrypted_quorum_key_shard: None,
            })
        }
    }
}
