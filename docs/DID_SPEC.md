# DID Method Specification — did:stellar

## Method Name
`stellar`

## Syntax
`did:stellar:<stellar-address>`

## Document Format
```json
{
  "@context": ["https://www.w3.org/ns/did/v1"],
  "id": "did:stellar:<address>",
  "verificationMethod": [{
    "id": "did:stellar:<address>#key-1",
    "type": "Ed25519VerificationKey2020",
    "controller": "did:stellar:<address>"
  }]
}
```
