# Gondolin

Gondolin is the secrets-management laboratory for Middle-earth. Its first
milestone is a SOPS-encrypted Kubernetes Secret reconciled by Flux.

This project deliberately does not use the normal user GPG keyring. Gondolin
has its own repository-local `GNUPGHOME`, its own public key, and its own
cluster Secret named `gondolin-sops-gpg`. The private key directory is ignored
by Git.

## One-time key creation

Requirements: `gpg`, `sops`, `kubectl`, and access to the target cluster.

```sh
make init-key
```

This creates:

- `.keys/gnupg/` — the dedicated private keyring; never committed
- `.keys/fingerprint` — the active fingerprint; never committed
- `keys/gondolin-sops.asc` — the public key; safe to commit

Back up the private key immediately:

```sh
make export-private-key > gondolin-sops-private.asc
chmod 600 gondolin-sops-private.asc
```

Move that export to a password manager or encrypted offline backup, then remove
the local export. Losing both `.keys/gnupg` and the backup makes committed
secrets unrecoverable.

## Create the example encrypted Secret

```sh
GONDOLIN_EXAMPLE_USERNAME=fellowship \
GONDOLIN_EXAMPLE_TOKEN='value-from-your-password-manager' \
make encrypt-example
```

Only `data` and `stringData` are encrypted. Names, namespaces, and resource
shape remain reviewable. Never commit a temporary plaintext manifest.

To inspect locally without writing plaintext:

```sh
make decrypt-example
```

## Install the dedicated Flux decryption key

```sh
make install-flux-key KUBECONFIG=/path/to/kubeconfig
```

The target exports only Gondolin's private key into
`flux-system/gondolin-sops-gpg`. It does not export the rest of any GPG
keyring.

Attach the Gondolin repository to Khazad-dûm using the example at
`../khazad-dum/apps/extensions/gondolin.yaml`. Its root reconciliation path is:

```text
./clusters/default
```

## Recreate or rotate the key

Before rotation, make sure the real plaintext values are recoverable from a
password manager or another authoritative secret store.

1. Export and retain the old private key until every secret has been migrated.
2. Move `.keys/gnupg` and `.keys/fingerprint` to a secure temporary backup.
3. Run `make init-key` to create a new isolated key.
4. Re-run each secret creation script with its authoritative values. This
   encrypts fresh manifests to the new fingerprint.
5. Run `make install-flux-key` to replace `gondolin-sops-gpg`.
6. Reconcile Gondolin and verify all Secrets and consumers.
7. Keep the old key backup until rollback is no longer required, then destroy
   it securely.

Do not append Gondolin's key to a shared Flux SOPS secret. Each future secrets
realm or trust boundary should use its own `secretRef`.

## Validate

```sh
make validate
```

