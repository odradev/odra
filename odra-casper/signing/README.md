# odra-casper-signing

Collects the signatures of one Casper transaction made on several machines, for accounts whose
action thresholds need more than one key (multisig).

The hash of a `TransactionV1` (and of a legacy `Deploy`) does not cover its approvals, so every
associated key can sign the same hash on its own machine, and the approvals are merged afterwards
without changing the hash.

- `read_transaction` / `write_transaction`: the transaction file, the `Transaction` as the node
  takes it (`{"Version1": {...}}`), pretty-printed JSON.
- `sign`: an approval of the transaction by one secret key.
- `SignatureString`: an approval as one line to paste into a chat,
  `odra-sig:<v1|deploy>:<transaction hash>:<public key>:<signature>`, all hex.
- `merge`: adds approvals to the transaction. The hash must match and every signature must verify.
- `inspect`: what the transaction does with the account it runs as: who signed, with what weight,
  which signers are not associated keys (they make it invalid), how much weight is missing for the
  `deployment` and `key_management` thresholds, and when it expires.
- `decode_account_change`: the `AccountChange` an account manager transaction makes, read from the
  transaction itself, so a signer reviews what they sign instead of a description they were sent.
- `build`: unsigned transactions (a native transfer as a `TransactionV1` or a legacy `Deploy`),
  since casper-types has no public builder.
- `SigningTransport`: where the transaction and its approvals are kept while they are collected.
  `FileTransport` keeps them in a directory (the offline flow); the CSPR.cloud awaiting deploy
  transport is planned (odradev/odra#726).
