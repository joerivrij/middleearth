# Erebor storage profiles

| Profile | Purpose |
| --- | --- |
| `bilbo` | Minimal, single-node Rook-managed Ceph with a development loop device. |
| `smaug` | External Ceph consumer: Ansible/cephadm owns the provider; Rook configures Kubernetes access. |
| `thorin` | Future complete Rook-managed deployment; currently a scaffold inheriting Bilbo. |

These names describe storage ownership and use, rather than small/medium/large
sizes. Thorin's inherited loop device must be replaced by independent raw disks
before its replica settings can provide meaningful redundancy.

`erebor/cluster/kustomization.yaml` selects a profile. Smaug is independent of
Bilbo and Thorin and contains no locally managed Ceph pools or object stores.
It requires provider credentials imported separately. See the
[Erebor guide](../../../../erebor/README.md) for provisioning and import steps.

Use a fresh Kubernetes consumer for Smaug: selecting external mode over an
existing local Rook deployment does not migrate its data.
