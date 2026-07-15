# Erebor

Erebor is a local k0s test cluster managed by Flux. It runs in an Apple
`container machine`, a persistent Linux VM backed by an OCI image.

## Create the cluster

Requirements: macOS 26, Apple container, Ansible, the Cilium CLI, kubectl, and
Flux. The
machine is provisioned over regular SSH; `container machine run` is not used.

Configure Apple's `.test` DNS domain once so `erebor.test` resolves, and make
sure your public key is present in `~/.ssh/authorized_keys`:

```bash
sudo container system dns create test
cd erebor/ansible
ansible-playbook site.yaml
cd ../..
export KUBECONFIG="$PWD/erebor/machine/kubeconfig"
./erebor/bootstrap-flux.sh
```

The playbook builds a Debian 13 systemd image, creates the persistent Apple
container machine, waits for OpenSSH, installs k0s through SSH, retrieves the
kubeconfig, and seeds Cilium so Flux can start on the custom-CNI cluster.

The bootstrap assumes the repository is public. No Git credentials or deploy
key are stored in the cluster. Commit the bootstrap manifests to `main` before
expecting reconciliation to become ready.

## Cluster overlays

- `bilbo` — “An Unexpected Journey”: the minimal, single-node development
  cluster.
- `thorin` — “The King Under the Mountain”: the standard homelab and HA
  deployment.
- `smaug` — “The Dragon’s Hoard”: large-scale storage and performance testing.

Bilbo corresponds to the previous small overlay, Thorin to medium, and Smaug to
large. See [the overlay guide](overlays/README.md) for the full
story and infrastructure mapping. Select an environment by changing `spec.path`
in `cluster/infrastructure.yaml` to the matching overlay.

The Ceph overlay still expects a raw `/dev/vdb` on every storage node. Apple
container machines currently provide the system disk only, so Rook will remain
degraded until a raw device is attached or the storage experiment is disabled.
