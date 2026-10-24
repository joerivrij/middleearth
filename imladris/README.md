# Imladris

Imladris provides reusable Ansible automation for Linux machines used by the
Middle-earth labs. It prepares hosts; it does not deploy applications.

## Responsibilities

- common packages and operator tooling
- users, SSH keys, and optional SSH hardening
- optional standalone containerd
- k0s controllers and workers
- optional Open vSwitch and OVN packages
- Raspberry Pi host preparation
- reusable VM and physical-host baselines
- local Linux machines through Lima or Apple Container Machine

Every role has defaults, uses Ansible modules where possible, and can be
selected independently. Lab-specific hosts, credentials, tokens, network
topology, and application configuration remain outside the roles.

## Layout

```text
imladris/
├── inventories/example/
├── playbooks/
│   ├── prepare.yml
│   ├── k0s.yml
│   ├── networking.yml
│   └── site.yml
└── roles/
    ├── common/
    ├── users/
    ├── ssh/
    ├── tooling/
    ├── raspberry_pi/
    ├── machine/
    ├── containerd/
    ├── k0s/
    └── ovs_ovn/
```

## Quick start

Requires Python 3.11+, Ansible Core 2.17+, and SSH access to the targets.

```sh
cp -R inventories/example inventories/erebor
$EDITOR inventories/erebor/hosts.yml
ansible-playbook -i inventories/erebor/hosts.yml playbooks/site.yml
```

Run only part of the machine preparation:

```sh
ansible-playbook -i inventories/erebor/hosts.yml playbooks/prepare.yml
ansible-playbook -i inventories/erebor/hosts.yml playbooks/k0s.yml
ansible-playbook -i inventories/erebor/hosts.yml playbooks/networking.yml
```

The example inventory deliberately leaves `imladris_users` empty. Add at least
one administrator with a public key before disabling SSH password
authentication.

## Choose compute, then install k0s

The `machine` role registers every target in `provisioned_machines`:

| `machine_backend` | Behavior |
| --- | --- |
| `baremetal` | Connect to an existing Linux host with `machine_address` and `machine_ssh_user`. |
| `lima` | Create/start a Lima VM and use its generated SSH configuration. |
| `container` | Build/create an Apple Container Machine and connect over SSH. |

VM creation runs locally on macOS. Bare metal can be managed from any Ansible
control host. The `machine_hosts` inventory entries describe the local
provisioner; put guest-specific variables in `machine_guest_vars`. Group-level
`all` variables also apply to the guests. Provisioners run serially because
Ansible's `add_host` action bypasses the normal host loop.

From `imladris/`:

```sh
# Only create/connect and prepare Linux.
make machines INVENTORY=inventories/example/hosts.yml

# Create/connect, prepare Linux, install single-node k0s, retrieve kubeconfig.
make machines-k0s K0S_INVENTORY=inventories/khazad-dum/hosts.yml  # Lima
make machines-k0s K0S_INVENTORY=inventories/apple/hosts.yml      # Apple
make machines-k0s K0S_INVENTORY=inventories/baremetal/hosts.yml  # existing host
```

Edit the bare-metal example's address and SSH user before running it. All
backends need working sudo in the guest. Apple expects the reusable keypair
`~/.ssh/id_apple_machine` and `~/.ssh/id_apple_machine.pub`. Only the public key
is staged in the ignored image build context.

`machines-k0s.yml` creates an independent single-node cluster on each selected
host. Use `machine_guest_vars` to give multiple hosts distinct cluster names
and output paths. For a controller/worker topology, use the existing
`k0s.yml` playbook with `k0s_controllers` / `k0s_workers` and join tokens.

`make single-node` remains an alias for the Lima profile. Its kubeconfig is
`machine-build/khazad-dum.kubeconfig`; run `make -C ../khazad-dum install`
afterward. The Apple profile writes the same path, so use one profile at a
time or override `kubeconfig_path`. Other inventories default to
`machine-build/<guest-inventory-name>.kubeconfig`.

The shared flow renders an address-aware k0s configuration with a custom CNI.
Set `k0s_node_address` for hosts with multiple interfaces and
`k0s_api_endpoint` for the operator's reachable API URL. A supplied
`k0s_config_source` is copied unchanged; its API certificate SANs must cover
the chosen endpoint. Cilium and Flux installation remain in Khazad-dûm.

Reusable Apple kernel/image sources live in `apple_machine/`. The Apple k0s
inventory selects that image with `machine_container_build_kernel: true`.
Rebuild/recreation are explicit options:

```sh
ansible-playbook -i inventories/apple/hosts.yml playbooks/machines-k0s.yml \
  -e kernel_rebuild=true -e machine_container_recreate=true
```

`make machines-reset INVENTORY=...` deletes configured VMs. It rejects bare
metal. Additional Lima disks are retained; normal provisioning never deletes
them. Changes to a Lima VM's disks/network definition require explicit VM
recreation; merely rendering a new definition does not update an existing VM.

## Erebor consumers

```sh
# Bilbo: Apple k0s host plus Erebor's loop-backed development OSD device.
make erebor

# Smaug: Linux hosts plus standalone Ceph; no k0s on the storage hosts.
make smaug
```

Bilbo uses `inventories/erebor`; Smaug uses `inventories/smaug`. Override
`EREBOR_INVENTORY` or `SMAUG_INVENTORY` for another backend/topology.
Erebor owns the Ceph roles and loop-device helper. Imladris composes them with
its machine preparation. Follow [Erebor's guide](../erebor/README.md) for the
optional Rook external-cluster consumer flow and Smaug network prerequisites.

## Host groups and variables

- `linux` receives the common, user, SSH, and tooling roles.
- `raspberry_pi` receives Pi-specific boot preparation.
- `k0s_controllers` and `k0s_workers` receive k0s.
- `ovs_ovn` receives Open vSwitch and OVN when explicitly enabled.

Important variables:

```yaml
imladris_users:
  - name: operator
    groups: [sudo]
    authorized_keys:
      - ssh-ed25519 AAAA...

ssh_password_authentication: false
k0s_version: v1.36.2+k0s.0
k0s_controller_enable_worker: true
k0s_worker_token: "{{ vault_k0s_worker_token }}"
containerd_install: false
ovs_ovn_install: true
raspberry_pi_prepare: true
```

Worker tokens and private material belong in Ansible Vault or an external
secret store. They must not be committed to this repository.

The k0s role uses k0s' bundled container runtime by default. Enable the
`containerd` role only for hosts that need a separately managed runtime.

## Idempotence and validation

```sh
make syntax
make lint
make check INVENTORY=inventories/example/hosts.yml
```

`make check` uses Ansible check mode. For meaningful idempotence testing, run a
playbook twice against an expendable VM and confirm the second run reports no
unexpected changes. Service installation commands are guarded by their
generated unit files, and configuration changes notify handlers.

## Scope boundary

Imladris may install and configure an operating-system package, container
runtime, k0s service, or host networking prerequisite. Kubernetes resources,
Helm releases, Flux reconciliation, and lab workloads belong in Khazad-dûm or
the individual lab repository.
