# Architecture — stellar-zkident

## System Overview

```
User → [Noir circuit] → ZK proof → credential_verifier → reputation_nft
                                          |
                                     did_registry
                                          |
                                    asp_registry (Merkle)
```

## Smart Contracts

| Contract | Role |
|---|---|
| did_registry | DID CRUD, did:stellar method, is_active() query |
| credential_verifier | ZK proof verification, has_credential() query |
| reputation_nft | Soulbound token, score tracking |
| asp_registry | Compliance set Merkle roots |

## ZK Circuits (Noir)

| Circuit | Proves | Private Input | Public Input |
|---|---|---|---|
| age_proof | age >= 18 | birth_year, month, day, salt | current_year, commitment |
| kyc_tier_proof | tier >= required | kyc_tier, salt | required_tier, commitment |
| membership_proof | Merkle membership | leaf_data, path, indices | merkle_root, leaf_commitment |
