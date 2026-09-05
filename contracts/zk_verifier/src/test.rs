#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

/// Build a mock VK of the correct byte length.
/// Layout: alpha(64) + beta(128) + gamma(128) + delta(128) + IC[0](64) + IC[1](64) = 576 bytes
fn mock_vk(env: &Env) -> Bytes {
    Bytes::from_array(env, &[0u8; 576])
}

/// Build a mock proof of the correct byte length.
/// Layout: A(64) + B(128) + C(64) = 256 bytes
fn mock_proof(env: &Env) -> Bytes {
    Bytes::from_array(env, &[0u8; 256])
}

#[test]
fn test_initialize_stores_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_get_vk_returns_stored_key() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    let vk = mock_vk(&env);
    client.initialize(&admin, &vk);
    assert_eq!(client.get_vk(), vk);
}

#[test]
fn test_update_vk_by_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    let new_vk = Bytes::from_array(&env, &[1u8; 576]);
    client.update_vk(&admin, &new_vk);
    assert_eq!(client.get_vk(), new_vk);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));
    client.initialize(&admin, &mock_vk(&env));
}

#[test]
fn test_verify_proof_rejects_all_zero_garbage() {
    // All-zero bytes decode to the point at infinity for both G1 and G2 (per
    // soroban-sdk's Bn254G1Affine/G2Affine docs). Pairing with infinity trivially
    // satisfies e(infinity, X) == 1 for any X, so without an explicit identity-element
    // check this would otherwise "verify" — see groth16::is_zero's doc comment. This
    // test exists specifically to pin that defensive check in place.
    let env = Env::default();
    env.mock_all_auths();
    let cid = env.register(ZkVerifierContract, ());
    let client = ZkVerifierContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &mock_vk(&env));

    let proof = mock_proof(&env);
    let inputs: Vec<BytesN<32>> = Vec::new(&env);
    let result = client.vrfy_prf(&proof, &inputs);
    assert!(!result, "an all-zero garbage proof must not pass real verification");
}

mod real_proof {
    //! Generates a genuine Groth16 BN254 proof with arkworks for a trivial
    //! "prove knowledge of x such that x*x = y" circuit, serializes it into the exact
    //! byte layout groth16::verify() expects, and confirms the contract accepts a real
    //! valid proof and rejects a tampered one.
    use super::*;
    use ark_bn254::{Bn254, Fq, Fq2, Fr as ArkFr, G1Affine, G2Affine};
    use ark_ff::{BigInteger, PrimeField};
    use ark_groth16::Groth16;
    use ark_relations::lc;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
    use ark_snark::SNARK;
    use ark_std::rand::{rngs::StdRng, SeedableRng};

    #[derive(Clone)]
    struct SquareCircuit {
        x: Option<ArkFr>,
        y: Option<ArkFr>,
    }

    impl ConstraintSynthesizer<ArkFr> for SquareCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<ArkFr>) -> Result<(), SynthesisError> {
            let x_var = cs.new_witness_variable(|| self.x.ok_or(SynthesisError::AssignmentMissing))?;
            let y_var = cs.new_input_variable(|| self.y.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(lc!() + x_var, lc!() + x_var, lc!() + y_var)?;
            Ok(())
        }
    }

    fn fq_to_be32(f: &Fq) -> [u8; 32] {
        let mut out = [0u8; 32];
        let bytes = f.into_bigint().to_bytes_be();
        out.copy_from_slice(&bytes);
        out
    }

    fn g1_to_bytes(env: &Env, p: &G1Affine) -> BytesN<64> {
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&fq_to_be32(&p.x));
        out[32..64].copy_from_slice(&fq_to_be32(&p.y));
        BytesN::from_array(env, &out)
    }

    fn fq2_to_be64(f: &Fq2) -> [u8; 64] {
        // Soroban's Bn254G2Affine Fp2 encoding is be(c1) || be(c0) — imaginary part first.
        let mut out = [0u8; 64];
        out[0..32].copy_from_slice(&fq_to_be32(&f.c1));
        out[32..64].copy_from_slice(&fq_to_be32(&f.c0));
        out
    }

    fn g2_to_bytes(env: &Env, p: &G2Affine) -> BytesN<128> {
        let mut out = [0u8; 128];
        out[0..64].copy_from_slice(&fq2_to_be64(&p.x));
        out[64..128].copy_from_slice(&fq2_to_be64(&p.y));
        BytesN::from_array(env, &out)
    }

    fn fr_to_bytes(env: &Env, f: &ArkFr) -> BytesN<32> {
        let mut out = [0u8; 32];
        let bytes = f.into_bigint().to_bytes_be();
        out.copy_from_slice(&bytes);
        BytesN::from_array(env, &out)
    }

    /// Runs real Groth16 setup + prove for x=3, y=9, and returns
    /// (vk_bytes, proof_bytes, public_input_bytes) in the exact layout the contract expects.
    fn build_real_proof(env: &Env) -> (Bytes, Bytes, Vec<BytesN<32>>) {
        let mut rng = StdRng::seed_from_u64(42);

        let setup_circuit = SquareCircuit { x: None, y: None };
        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(setup_circuit, &mut rng)
            .expect("groth16 setup should succeed for this trivial circuit");

        let x = ArkFr::from(3u64);
        let y = ArkFr::from(9u64);
        let prove_circuit = SquareCircuit { x: Some(x), y: Some(y) };
        let proof = Groth16::<Bn254>::prove(&pk, prove_circuit, &mut rng)
            .expect("proving should succeed for a satisfied circuit");

        // Sanity-check against arkworks' own verifier before trusting our own serialization.
        assert!(
            Groth16::<Bn254>::verify(&vk, &[y], &proof).expect("arkworks verify should not error"),
            "arkworks' own verifier rejected the proof we just generated — setup/prove bug, not a serialization bug"
        );

        assert_eq!(vk.gamma_abc_g1.len(), 2, "expected IC[0] + one IC per public input");

        let mut vk_bytes = Bytes::new(env);
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.alpha_g1)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.beta_g2)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.gamma_g2)));
        vk_bytes.append(&Bytes::from(g2_to_bytes(env, &vk.delta_g2)));
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.gamma_abc_g1[0])));
        vk_bytes.append(&Bytes::from(g1_to_bytes(env, &vk.gamma_abc_g1[1])));

        let mut proof_bytes = Bytes::new(env);
        proof_bytes.append(&Bytes::from(g1_to_bytes(env, &proof.a)));
        proof_bytes.append(&Bytes::from(g2_to_bytes(env, &proof.b)));
        proof_bytes.append(&Bytes::from(g1_to_bytes(env, &proof.c)));

        let mut public_inputs: Vec<BytesN<32>> = Vec::new(env);
        public_inputs.push_back(fr_to_bytes(env, &y));

        (vk_bytes, proof_bytes, public_inputs)
    }

    #[test]
    fn test_verify_accepts_a_real_valid_groth16_proof() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let (vk_bytes, proof_bytes, public_inputs) = build_real_proof(&env);
        client.initialize(&admin, &vk_bytes);

        let result = client.vrfy_prf(&proof_bytes, &public_inputs);
        assert!(result, "a genuine, correctly-serialized Groth16 proof should verify");
    }

    #[test]
    fn test_verify_rejects_a_proof_for_the_wrong_public_input() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let (vk_bytes, proof_bytes, _correct_inputs) = build_real_proof(&env);
        client.initialize(&admin, &vk_bytes);

        // Claim y=10 instead of the real y=9 the proof was actually generated for.
        let wrong_y = ArkFr::from(10u64);
        let mut wrong_inputs: Vec<BytesN<32>> = Vec::new(&env);
        wrong_inputs.push_back(fr_to_bytes(&env, &wrong_y));

        let result = client.vrfy_prf(&proof_bytes, &wrong_inputs);
        assert!(!result, "a proof must not verify against a public input it wasn't generated for");
    }
}

/// Verifies this contract's `vrfy_prf()` against VKs and proofs generated by an actual,
/// complete Groth16 trusted-setup pipeline (circom -> snarkjs powers-of-tau -> phase 2 ->
/// contribution -> export) for this project's real `age_proof.circom`,
/// `kyc_tier_proof.circom`, and `membership_proof.circom` circuits — not a toy circuit, and
/// not hand-crafted bytes. Each proof/VK pair was independently confirmed valid by
/// `snarkjs groth16 verify` before being converted to this contract's byte layout, so a
/// pass here is real evidence the byte conversion (endianness, G2 c1||c0 ordering, IC
/// layout) is actually correct end to end, not just internally self-consistent. See
/// circuits/convert_to_soroban.mjs for how these bytes were produced from
/// circuits/build/{age_proof,kyc_tier_proof,membership_proof}/*_vk.json and proof.json.
mod real_zkident_circuits {
    use super::*;
    use soroban_sdk::{TryFromVal, Val};

    fn hex_to_bytes(env: &Env, hex: &str) -> Bytes {
        fn nibble(c: u8) -> u8 {
            match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => panic!("invalid hex digit"),
            }
        }
        let hex_bytes = hex.as_bytes();
        let mut out = Bytes::new(env);
        let mut i = 0;
        while i + 1 < hex_bytes.len() {
            let byte = (nibble(hex_bytes[i]) << 4) | nibble(hex_bytes[i + 1]);
            out.push_back(byte);
            i += 2;
        }
        out
    }

    fn hex_to_scalar(env: &Env, hex: &str) -> BytesN<32> {
        let bytes: Val = hex_to_bytes(env, hex).into();
        BytesN::<32>::try_from_val(env, &bytes).unwrap()
    }

    // circuits/build/age_proof/age_proof_vk.hex / _proof.hex / _public_inputs.json, for a
    // real demo subject born 2000-06-15 (age 26), checked against current_year=2026.
    // Public inputs are [valid=1, current_year, credential_commitment] — Circom orders the
    // circuit's own `output` signal before its declared `public` inputs.
    const AGE_PROOF_VK_HEX: &str = "2c9d02504b330c2af05244a56931dd63d1bb1a0e82ebd1ab84c97752dec2ea0a23b17cea67907fb0690e37f3315d2b18ccd798af1072294f742a985c5c43cafe25b03a115467194097eda52181fc0d07a47d4c50a91a3087ada19ab90c4adc3a1a0f773917de5b6a1442fd0b4379cec74dfa8f3dde60a3e4c5176b100d1b79e62609925cd946280b6f4ad76270b94152f0ddc081a07d51760324429ae2a1bf960fe8c5bf3785b5dc838c73edd0461ee1f4b76d157d1c0c4a1bbbb025cccddc20198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa1553f83bfc3efb110e782e205e2aa5c7e0985daf3393ad059228b485090c66792f285f0b2cb8fbbd6cd0a98ee52ce132cbc2a589e72a755ffc71e2f59a6e2d1e071084c42db273630f7470fa221b67d65b8c4db8e41709de508ee624eaa51b3e17068f679709777cba3542b112c50a1d4bf3ba5730714030b30dbf4d14fbf3321218f4a832a8d1be44f86c464d7646ba0fa8a702ecee685006e6631c082f845a05a5b1843fd4ed6bb0ace3dc0d9b45daeda285de9ab6ce912f100f407240c20a1774294f5c9630523ec026be412335b8ad616e5e2cc708ba6ff09dbae6b59ef10b507c3367592f11a53d0518a6fa479dd43d806e208249d3d4ec57854c6293e2191e95eccf58d8080eb33c5c3fb837523ad8786a2429da3ec3edc76e33e145b22a2c54cac750b1b9f456b8efee8e634988b8809c4ef623dd304cdc991636141d07864357a75bab1826cd6770d4e66a18edbfd238166b71277ddc90fe730af88f04fa4b51df299a05b59491ec86de161783faaeef80a82c2240d8449f8de4ee86";
    const AGE_PROOF_PROOF_HEX: &str = "21f23d8edab6839255c68a400fa5395daef0cb99217bda635c25c57e9fe018892fd0d885298d8ece9590e16c8775a268d1ef32d61e47cc62268a4022410ad4762f23db3ea6ee257d12a334b54b3f3e37d23eddc57ad480174ee0dd727b931d9b2f1fe92ac88487c716410c163b1f819bcbd5a13a23cd4f01948fa38024183bbe14a31ba5d99f6c4f6866434ac4b5989e78903d8633cceae5634513f13f82067a0c3512509b2fbef860344b29331145dbaac30ac796c860924a7bc586798795df16724f0a087ae5b345ff384cb994aa8acdb8f982b7ae97e86960625ed05deb3126334a640ee2257af7bbf44323f81fc61082ebaa151cde8731ebdae70e5aeb57";
    const AGE_PROOF_PUBLIC_INPUTS_HEX: [&str; 3] = [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "00000000000000000000000000000000000000000000000000000000000007ea",
        "1ff5cd88ee3ae6617cb6f4bae67387d1d59048af78460be38852c98dae0c747f",
    ];

    // circuits/build/kyc_tier_proof/kyc_tier_proof_vk.hex / _proof.hex /
    // _public_inputs.json, for a real demo subject at tier 3, checked against a required
    // tier of 2. Public inputs: [valid=1, required_tier, credential_commitment].
    const KYC_TIER_VK_HEX: &str = "2c9d02504b330c2af05244a56931dd63d1bb1a0e82ebd1ab84c97752dec2ea0a23b17cea67907fb0690e37f3315d2b18ccd798af1072294f742a985c5c43cafe25b03a115467194097eda52181fc0d07a47d4c50a91a3087ada19ab90c4adc3a1a0f773917de5b6a1442fd0b4379cec74dfa8f3dde60a3e4c5176b100d1b79e62609925cd946280b6f4ad76270b94152f0ddc081a07d51760324429ae2a1bf960fe8c5bf3785b5dc838c73edd0461ee1f4b76d157d1c0c4a1bbbb025cccddc20198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa1ff8becc321c02d20ba4335e9114c02f87a47899815166c960a5e9d460c395a922b9c3b20b83b46e4c6797a076b8aa03adee01c5dd42e7bd94ff4b2e3db2d3d42a74df312e741baa34e3f944857954edba49c1764969f5a9ed4b2958b84f5f2201b53d16f6d6ef8a6d1ed4e34763abddcdfd1e4878a747544bea84f0ebde0a870509aa06ce65dbb220988323140b075eca445fc94e659ec88685537214d1252f09b75665f1f719394a0f36787850772db03ab248433ea1099d6a58222072de381e2532ffb8380f129772b947c1d23545297ff6c5cd985347757f682d4125b96412ac315a6cf2b1566bb1771a3eedb663c69cb98a044d477800888fc13dc3ec100f6e5e32bb3d53f224d5a0e81bd190676956e2b0a0fc6a551b747f4993116ad530022657a800619ae50b0976388a803b7f4e6ddccf56aa374908048cfd2ab0102c9597004461687ac5a6aeafbac1829ffeee3e696889e0757e706f3ba4066ba4256e5389f3b87f1f5fbe78cedfb19584ee635d68ced521a69ad5903a9bfb0a42";
    const KYC_TIER_PROOF_HEX: &str = "0b1716ee1075beb6b8d4bdc9aa832f10d4d2bbb0315b78ec969d2bdf662348b40a40b190ae3e87ed508a07d0b36bb48f0566f900bdfc152a1e08ac98a53edb832799fe32740f7d3a140773fc1b59c10cc4fce06c138c5251ddfdd2c6d0111fb9293f3e4db3d7a3fcfb52865b699a07486452f5dcdd858720f935fec64a572c53036dc54a083f13950b1c89a389a9007541de470355668192b0390636b7c7432411351ce4aab1f404025444bd143fbf0638bd90d0ce594e9bee799a621cfec0cb0f3c561ce114589c7040dbbf1557c2dbc75312798c2998022e49ecb2c8844bbb0b02f5ef5f2b03545b34ecd8cd2d5aa4d66bd0de224584698766b52f2b378f5e";
    const KYC_TIER_PUBLIC_INPUTS_HEX: [&str; 3] = [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
        "147ce224e1becfd73d544456bf40cf188fc0c13e1e08311892c4a43807b61c64",
    ];

    // circuits/build/membership_proof/membership_proof_vk.hex / _proof.hex /
    // _public_inputs.json, for a real leaf at index 0 of a real depth-20 Merkle tree.
    // Public inputs: [valid=1, merkle_root, leaf_commitment].
    const MEMBERSHIP_VK_HEX: &str = "2c9d02504b330c2af05244a56931dd63d1bb1a0e82ebd1ab84c97752dec2ea0a23b17cea67907fb0690e37f3315d2b18ccd798af1072294f742a985c5c43cafe25b03a115467194097eda52181fc0d07a47d4c50a91a3087ada19ab90c4adc3a1a0f773917de5b6a1442fd0b4379cec74dfa8f3dde60a3e4c5176b100d1b79e62609925cd946280b6f4ad76270b94152f0ddc081a07d51760324429ae2a1bf960fe8c5bf3785b5dc838c73edd0461ee1f4b76d157d1c0c4a1bbbb025cccddc20198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa0f5b0cdfc3a89d7423a5fe233deee24e7d7dfeb71dac96bae343d83703b3d2410cf95b692e37384564a07849bf3f184859f1c2a53337c7a0d2ed9738ab50e1d324b10ddf80efec8a42ef0312c30a2b6c4067f693f0c0b7b1618f00d2eb51074c0d07e9537b1ce2d3af665d5f1019901625e4a6d130a7daa6a7bd15fc0bb8edbf0751ae75720b4e0425b44f308169ec92755c0d07fac0c06094ab0aa527ca56bd2d29470bdf401c0b19fa6d6e1073be127b3fbb784b2d979d1da40203af3d27fc062746f0616f91a39d4bfa2f508cff534f284041afb0b778507f4d48b84ac42c025a60bea018f6726a1eceb7a37de074968c97024564068c13ff0279882cd3d015d9d77518744ea73ab08e1ce27a249263b65ab11c20dab9608ead22ae5ec2970e43bfd484637136ddaccc12dbae39cee184ec59a01c6dd06480534b671507602dceb81b33f5d55c839e67bfdf3abe35bbcd71d7fa47cfc4b68bf323881fb03e10702a319ca530a48fb25cb360dcd7801f71bc8cf5c5d408493dc61065f1a9db";
    const MEMBERSHIP_PROOF_HEX: &str = "2564e2a463eca406837fb0edf630c55c7a7af45e03a50de5f44479d5270082ea22b4b5a289c1cb79c0491b9f885aa69f05d542569abed6b14352d9583d33ede114b9044bf0d6bfc68d7ce6a88733637ed6d537e97f4793631e5700b31da1ac5418a5bae2c9baf1d26fa36fb5cc7f1513dd7e6f7ae9b59a3ecc99c475620cc005179f8501de6fe720966dfa52bfb595542bd1c82140974fbeebfbdbbb7f88e62f06ab98ec0ef362257de2f6af05f757f310e2f19ba2fb11cea381bc95a6b75463264338d746fd991efbc50a4bca8442c8aff6ae4318170b87806a5ceada930db929b9a9543b528c966414d524fbd4d578c589b30de72178d5f8228c2a7b105d78";
    const MEMBERSHIP_PUBLIC_INPUTS_HEX: [&str; 3] = [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0d1a9198fa2fdfb1e646029490be67e4ad59c4a19fefab716aff1303b17532c6",
        "2465b55956e84da7fa95ff166a069e3ee551093a7a59a27d70a3867b4a8e51c0",
    ];

    fn assert_verifies(vk_hex: &str, proof_hex: &str, public_inputs_hex: &[&str]) -> bool {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, vk_hex);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, proof_hex);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in public_inputs_hex {
            inputs.push_back(hex_to_scalar(&env, h));
        }

        client.vrfy_prf(&proof, &inputs)
    }

    #[test]
    fn test_verify_accepts_the_real_age_proof_circuit() {
        let result = assert_verifies(AGE_PROOF_VK_HEX, AGE_PROOF_PROOF_HEX, &AGE_PROOF_PUBLIC_INPUTS_HEX);
        assert!(result, "a real proof for the real age_proof circuit, against its own real VK, must verify");
    }

    #[test]
    fn test_verify_accepts_the_real_kyc_tier_proof_circuit() {
        let result = assert_verifies(KYC_TIER_VK_HEX, KYC_TIER_PROOF_HEX, &KYC_TIER_PUBLIC_INPUTS_HEX);
        assert!(result, "a real proof for the real kyc_tier_proof circuit, against its own real VK, must verify");
    }

    #[test]
    fn test_verify_accepts_the_real_membership_proof_circuit() {
        let result = assert_verifies(MEMBERSHIP_VK_HEX, MEMBERSHIP_PROOF_HEX, &MEMBERSHIP_PUBLIC_INPUTS_HEX);
        assert!(result, "a real proof for the real membership_proof circuit, against its own real VK, must verify");
    }

    #[test]
    fn test_verify_rejects_the_real_age_proof_against_a_tampered_current_year() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, AGE_PROOF_VK_HEX);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, AGE_PROOF_PROOF_HEX);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in AGE_PROOF_PUBLIC_INPUTS_HEX {
            inputs.push_back(hex_to_scalar(&env, h));
        }
        // Claim a different current_year than the one this proof was actually generated for.
        inputs.set(1, hex_to_scalar(&env, "0000000000000000000000000000000000000000000000000000000000000834"));

        let result = client.vrfy_prf(&proof, &inputs);
        assert!(!result, "a real proof must not verify against public inputs it wasn't generated for");
    }

    #[test]
    fn test_verify_rejects_the_real_membership_proof_against_a_tampered_root() {
        let env = Env::default();
        env.mock_all_auths();
        let cid = env.register(ZkVerifierContract, ());
        let client = ZkVerifierContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        let vk = hex_to_bytes(&env, MEMBERSHIP_VK_HEX);
        client.initialize(&admin, &vk);

        let proof = hex_to_bytes(&env, MEMBERSHIP_PROOF_HEX);
        let mut inputs: Vec<BytesN<32>> = Vec::new(&env);
        for h in MEMBERSHIP_PUBLIC_INPUTS_HEX {
            inputs.push_back(hex_to_scalar(&env, h));
        }
        // Claim a fabricated root instead of the one this proof was actually generated for.
        inputs.set(1, hex_to_scalar(&env, "0000000000000000000000000000000000000000000000000000000000000001"));

        let result = client.vrfy_prf(&proof, &inputs);
        assert!(!result, "a real proof must not verify against a merkle_root it wasn't generated for");
    }
}
