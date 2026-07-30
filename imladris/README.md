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

## Create local machines on macOS

The `machine` role supports two interchangeable backends:

- `machine_backend: lima` renders a Debian 13 Lima definition, creates or
  starts it with `limactl`, and reuses Lima's generated SSH configuration.
- `machine_backend: container` builds a minimal Debian 13 systemd image,
  creates an Apple Container Machine, discovers its address, and connects with
  the configured SSH keypair.

Both backends register their guest in the same temporary
`provisioned_machines` group. The second play then applies the normal `common`,
`users`, `ssh`, and `tooling` roles, so guest preparation does not depend on
the hypervisor.

```sh
# Create and prepare both example backends.
make machines

# Or select one backend.
make machines-lima
make machines-container

# Explicitly delete the configured machines.
make machines-reset
```

Apple Container Machine expects `~/.ssh/id_apple_machine` and its `.pub` file
by default. Override `machine_container_ssh_private_key` when using another
keypair. Machine deletion is never part of normal preparation and only occurs
through `machines-reset`.

## Erebor profile

Erebor is now a consumer of Imladris rather than maintaining a separate
Ansible tree. Its inventory selects Apple Container Machine and points the
generic machine and k0s roles at Erebor's custom kernel, system image, and k0s
configuration:

```sh
make erebor
```

The lab-specific kernel and image assets remain under `../erebor`; the
provisioning workflow, machine lifecycle, k0s installation, and kubeconfig
generation live here. Platform bootstrap continues in Khazad-dûm.

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
