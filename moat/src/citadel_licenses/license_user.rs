// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::api::{MoatContext, MoatCore};
use crate::{
    Error, LicenseCircuit, PayloadSender, LICENSE_CONTRACT_ID,
    USE_LICENSE_METHOD_NAME,
};
use crate::{ARITY, DEPTH};
use bytecheck::CheckBytes;
use dusk_bls12_381::BlsScalar;
use dusk_jubjub::JubJubScalar;
use dusk_pki::PublicSpendKey;
use dusk_plonk::prelude::Proof;
use poseidon_merkle::Opening;
use rand::rngs::OsRng;
use rkyv::{Archive, Deserialize, Serialize};
use zk_citadel::license::{CitadelProverParameters, License, SessionCookie};

/// Use License Argument.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct UseLicenseArg {
    pub proof: Proof,
    pub public_inputs: Vec<BlsScalar>,
}

pub struct LicenseUser;

impl LicenseUser {
    #[allow(clippy::too_many_arguments)]
    /// Calculates and verified proof, sends proof along with public parameters
    /// as arguments to the license contract's use_license method.
    /// Returns transaction id and a session cookie.
    pub async fn prove_and_use_license(
        moat_context: &MoatContext,
        psk_lp: &PublicSpendKey,
        psk_sp: &PublicSpendKey,
        license: &License,
        opening: Opening<(), DEPTH, ARITY>,
        rng: &mut OsRng,
        challenge: &JubJubScalar,
    ) -> Result<(BlsScalar, SessionCookie), Error> {
        let (_psk_user, ssk_user) = MoatCore::get_wallet_keypair(moat_context)?;
        let (cpp, sc) = CitadelProverParameters::compute_parameters(
            &ssk_user, license, psk_lp, psk_sp, challenge, rng, opening,
        );
        let circuit = LicenseCircuit::new(&cpp, &sc);

        let (proof, public_inputs) = moat_context
            .prover
            .prove(rng, &circuit)
            .expect("Proving should succeed");

        assert!(!public_inputs.is_empty());

        moat_context
            .verifier
            .verify(&proof, &public_inputs)
            .expect("Verifying the circuit should succeed");

        let use_license_arg = UseLicenseArg {
            proof,
            public_inputs,
        };

        let tx_id = PayloadSender::execute_contract_method(
            use_license_arg,
            moat_context,
            LICENSE_CONTRACT_ID,
            USE_LICENSE_METHOD_NAME,
        )
        .await?;
        Ok((tx_id, sc))
    }
}
