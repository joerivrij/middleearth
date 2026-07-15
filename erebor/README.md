# Erebor

Erebor is a local k0s test cluster managed by Flux. It runs in an Apple
`container machine`, a persistent Linux VM backed by an OCI image.

## Create the cluster

Requirements: macOS 26, Apple container, Ansible, the Cilium CLI, kubectl, and
Flux. The
machine is provisioned over regular SSH; `container machine run` is not used.

Create one reusable Apple-machine keypair on macOS, similar to a key shared by
a fleet of Raspberry Pis:

```bash
ssh-keygen -t ed25519 -a 100 \
  -f ~/.ssh/id_apple_machine \
  -C "apple-container-machines"
chmod 600 ~/.ssh/id_apple_machine
chmod 644 ~/.ssh/id_apple_machine.pub
```

Keep the private key only in `~/.ssh` and back it up securely. The playbook
requires this keypair, stages only `id_apple_machine.pub` into the ignored image
build context, and bakes that public key into every machine image. It uses
`~/.ssh/id_apple_machine` for Ansible connections and discovers each machine's
IP automatically. Neither Apple `.test` DNS nor your personal
`authorized_keys` is required.

```bash
cd erebor/ansible
ansible-playbook site.yaml
cd ../..
export KUBECONFIG="$PWD/erebor/machine/kubeconfig"
./erebor/bootstrap-flux.sh
```

The playbook first builds `kernel/Image`, using Apple's current machine kernel
configuration as its baseline and adding the socket and TPROXY netfilter
features required by Cilium's L7 proxy plus the eBPF JIT required by Cilium's
datapath. It then builds a Debian 13 systemd
image, creates the persistent Apple container machine with that kernel, waits
for OpenSSH, installs k0s through SSH, retrieves the kubeconfig, and seeds
Cilium (including Envoy) so Flux can start on the custom-CNI cluster. The
generated `kernel/base.config` and `kernel/Image` are intentionally ignored by
Git.

The first kernel build takes a few minutes. Apple container's BuildKit machine
may need more resources than its default for this; for example:

```bash
container builder stop
container builder delete
container builder start --cpus 6 --memory 8G
```

The image also makes the guest VM's root mount recursively shared before k0s
starts, which Cilium requires for BPF and cgroup v2 mount propagation.

When the machine image, custom kernel, or shared Apple-machine SSH key changes,
recreate an existing machine explicitly:

```bash
ansible-playbook site.yaml -e machine_recreate=true
```

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
