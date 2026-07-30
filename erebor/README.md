# Erebor

Erebor is the Ceph and storage laboratory for Middle-earth. Its first
environment is a local k0s test cluster managed by Flux, running in an Apple
`container machine` backed by an OCI image.

Erebor owns storage experiments and their machine-specific assets. Reusable
machine provisioning belongs in Imladris, while the shared Kubernetes platform
and Erebor's optional deployment overlays belong in Khazad-dûm.

## Create the cluster

Requirements: macOS 26, Apple container, Ansible, the Cilium CLI, kubectl, and
Flux. Imladris provisions the machine over regular SSH; `container machine
run` is not used.

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
make -C imladris erebor
make -C khazad-dum bootstrap-erebor ENV=bilbo \
  KUBECONFIG=../erebor/machine/kubeconfig
```

The Imladris Erebor profile first builds `kernel/Image`, using Apple's current machine kernel
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
ansible-playbook -i imladris/inventories/erebor/hosts.yml \
  imladris/playbooks/erebor.yml -e machine_container_recreate=true
```

Rook on Bilbo uses a 10 GiB sparse file attached as `/dev/loop0`, because Apple
container machines do not currently expose a secondary-disk attachment option.
The custom kernel includes loop and device-mapper support, and the Rook operator
is explicitly configured to accept loop devices. The kernel build also stages
its matching loadable RBD and Ceph modules into the Debian machine image so
Ceph-CSI can load `rbd.ko`. This is test storage: it shares the VM's root disk
and is not an HA or production Ceph layout.

After changing the kernel configuration, rebuild it and recreate the VM:

```bash
ansible-playbook -i imladris/inventories/erebor/hosts.yml \
  imladris/playbooks/erebor.yml \
  -e kernel_rebuild=true \
  -e machine_container_recreate=true
```

The bootstrap assumes the repository is public. No Git credentials or deploy
key are stored in the cluster. Commit the bootstrap manifests to `main` before
expecting reconciliation to become ready.

## Shared automation and platform

Erebor is now a consumer of the two reusable foundations:

```text
imladris/
├── inventories/erebor/          # Erebor machine profile
└── playbooks/erebor.yml

khazad-dum/
├── infrastructure/base/         # shared Kubernetes platform
├── apps/erebor/                  # optional Erebor storage and overlays
└── clusters/erebor/              # Bilbo, Thorin, and Smaug composition
```

Imladris owns machine lifecycle, reusable Linux roles, k0s installation, and
kubeconfig generation. Erebor keeps only its custom kernel and machine-image
assets. Khazad-dûm owns Flux, Cilium, cert-manager, Traefik, metrics-server, and
the optional Rook/Ceph addition selected by Erebor.

```bash
make -C khazad-dum bootstrap-erebor ENV=bilbo
make -C khazad-dum bootstrap-erebor ENV=thorin
make -C khazad-dum bootstrap-erebor ENV=smaug
```

## Cluster overlays

- `bilbo` — “An Unexpected Journey”: the minimal, single-node development
  cluster.
- `thorin` — “The King Under the Mountain”: the standard homelab and HA
  deployment.
- `smaug` — “The Dragon’s Hoard”: large-scale storage and performance testing.

Bilbo corresponds to the previous small overlay, Thorin to medium, and Smaug to
large. See the
[overlay guide](../khazad-dum/apps/erebor/overlays/README.md) for the full story
and infrastructure mapping. The selected Khazad-dûm cluster entry point
references its matching add-on overlays.

Thorin and Smaug currently inherit Bilbo's loop-backed test device. Their HA
replica settings describe the intended future multi-node topology; they require
real, independent raw disks before they can provide meaningful redundancy.
