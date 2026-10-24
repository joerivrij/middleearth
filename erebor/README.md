# Erebor

Erebor owns Ceph and storage experiments. Imladris creates or connects to Linux
hosts; Khazad-dûm supplies the Kubernetes platform and optional Rook overlays.

| Profile | Storage ownership | Status |
| --- | --- | --- |
| Bilbo | Rook manages Ceph inside Kubernetes. | Minimal single-node development lab. |
| Smaug | Ansible bootstraps standalone Ceph with cephadm; Kubernetes optionally consumes it through Rook. | New flow, awaiting live testing. |
| Thorin | Rook manages a complete multi-node Ceph deployment. | Future design; current overlay is a scaffold inheriting Bilbo. |

```text
imladris/roles/machine/       # baremetal, Lima, Apple Container Machine
imladris/apple_machine/      # reusable Apple kernel and Linux system image
imladris/playbooks/          # compose machine preparation with k0s or Ceph
erebor/roles/ceph/           # standalone Ceph bootstrap, hosts, OSDs, RBD pool
erebor/roles/ceph_loop/      # Bilbo-only loop device preparation
erebor/machine/             # Bilbo loop-device helper and service
erebor/cluster/             # selects the Kubernetes consumer profile
erebor/clusters/            # Bilbo, Smaug, Thorin Flux compositions
khazad-dum/apps/erebor/      # Rook operator and cluster overlays
```

## Bilbo: simple Rook-managed Ceph

Requires macOS with Apple Container Machine, Ansible, and the shared SSH
keypair. Create it once if needed:

```sh
ssh-keygen -t ed25519 -a 100 -f ~/.ssh/id_apple_machine -C apple-container-machines
```

From the repository root:

```sh
make -C imladris erebor
make -C khazad-dum install \
  SINGLE_NODE_KUBECONFIG=../imladris/machine-build/erebor.kubeconfig
```

Select `../clusters/bilbo` in `erebor/cluster/kustomization.yaml` and enable
`azanulbizar/erebor.yaml` in `khazad-dum/clusters/khazad-dum/kustomization.yaml`.
Commit and push these manifests before expecting Flux to reconcile them.

Bilbo uses a 10 GiB sparse file attached as `/dev/loop0`. It shares the guest
root disk and provides no independent redundancy. The loop helper is installed
by Erebor after k0s provisioning; it is no longer baked into the reusable
Apple image. The custom kernel retains loop, device-mapper, RBD/Ceph modules,
and Cilium networking support. Generated kernel/module artifacts remain ignored.

The first kernel build can require a larger Apple builder, for example
`container builder start --cpus 6 --memory 8G` after stopping/deleting the old
builder. Rebuild the kernel and recreate a disposable Bilbo machine explicitly:

```sh
cd imladris
ansible-playbook -i inventories/erebor/hosts.yml playbooks/erebor.yml \
  -e kernel_rebuild=true -e machine_container_recreate=true
```

Old paths `erebor/kernel`, `erebor/machine/Containerfile`, and
`khazad-dum/apple_machine` are superseded by `imladris/apple_machine`.
Kubeconfigs now live in `imladris/machine-build/`.

## Smaug: standalone Ceph

The default example creates a Debian Lima VM and two independent 20 GiB virtual
OSD disks. It bootstraps Ceph 19.2.3 with cephadm, applies explicit OSD device
specifications, and creates the `smaug-rbd` pool with two replicas on one host.
This is a development topology, not host-level HA. No k0s is installed here.

Before running:

1. Configure Lima's `shared` socket_vmnet network following the
   [Lima VM networking guide](https://lima-vm.io/docs/config/network/vmnet/).
   The inventory selects `lima: shared` on `lima0`. The default `vzNAT` network
   does not permit guest-to-guest access.
2. Put the Kubernetes consumer VMs on the same reachable network. In their
   inventory, set `machine_lima_networks: [{lima: shared, interface: lima0}]`
   before creating them. External consumers need routes to all monitor and
   OSD addresses; forwarding only the Kubernetes API does not provide this.
3. Review `imladris/inventories/smaug/hosts.yml`, especially
   `machine_guest_vars.ceph_osd_devices`. These disks will be used for Ceph.
   The example expects `/dev/vdb` and `/dev/vdc`; confirm names with `lsblk`
   when changing the VM configuration.

```sh
# Optional first step: create only, then inspect disks with limactl shell smaug lsblk.
make -C imladris machines INVENTORY=inventories/smaug/hosts.yml
make -C imladris smaug
limactl shell smaug sudo cephadm shell -- ceph -s
```

The role never selects all available disks or zaps devices. Ceph requires
unused raw devices. Lima disks are created with formatting disabled and are
retained when the VM is reset; deleting/recreating a VM is not a Ceph data
reset. A fresh bootstrap against old OSD disks requires a deliberate recovery
or separate disk cleanup workflow.

For existing bare-metal hosts, use the Imladris bare-metal inventory pattern
and set per-host `ceph_osd_devices` and `ceph_address` in `machine_guest_vars`.
Set `ceph_address_interface: ""` when not using Lima's `lima0`. For multiple
hosts, set `ceph_single_host: false` and choose `ceph_pool_size` /
`ceph_pool_min_size` for that topology. Each host needs a unique hostname and
mutually reachable SSH/Ceph addresses. The first registered host bootstraps
Ceph and distributes its orchestration public key to the remaining hosts.
Smaug's role currently supports Debian hosts.

Apple machines can use the same preparation flow, but need usable raw OSD
storage supplied separately; Bilbo's loop device is not implicitly enabled
for Smaug.

## Optional Smaug consumer on k0s

Create the consumer independently with Imladris and install Khazad-dûm:

```sh
make -C imladris machines-k0s K0S_INVENTORY=inventories/khazad-dum/hosts.yml
make -C khazad-dum install
```

On a fresh consumer, select `../clusters/smaug` in
`erebor/cluster/kustomization.yaml`, then enable `azanulbizar/erebor.yaml` in
the Khazad-dûm cluster. Commit/push and reconcile Flux. Smaug deploys its own
external-mode HelmRelease and disables loop-device support. It inherits no
Bilbo/Thorin pools, object stores, local OSDs, or ingress routes.

Switching an existing Bilbo/Thorin deployment to Smaug is not a data migration:
Flux/Helm can prune the old local storage resources. Use a fresh consumer for
this flow and migrate existing data separately.

Rook still needs the provider connection data and scoped CSI credentials.
Follow the version-matched Rook
[provider export](https://rook.io/docs/rook/v1.19/CRDs/Cluster/external-cluster/provider-export/)
and [consumer import](https://rook.io/docs/rook/v1.19/CRDs/Cluster/external-cluster/consumer-import/)
steps. Use the scripts from the `v1.19.5` Rook checkout to match the Helm chart.

On the provider, place `deploy/examples/create-external-cluster-resources.py`
from that checkout in `/root/rook-export/`, then run as root:

```sh
umask 077
cephadm shell --mount /root/rook-export -- \
  python3 /mnt/rook-export/create-external-cluster-resources.py \
  --rbd-data-pool-name smaug-rbd --namespace rook-ceph \
  --k8s-cluster-name khazad-dum --restricted-auth-permission \
  --format bash > /root/smaug-external.env
```

Securely copy the exported file to the operator machine. From a Bash shell
with the consumer kubeconfig selected, use Rook's matching import script:

```sh
export KUBECONFIG=/absolute/path/to/imladris/machine-build/khazad-dum.kubeconfig
export NAMESPACE=rook-ceph
source /secure/path/smaug-external.env
source /path/to/rook-v1.19.5/deploy/examples/import-external-cluster.sh
kubectl -n rook-ceph get cephcluster
kubectl get storageclass ceph-rbd
```

The import creates connection secrets and the `ceph-rbd` StorageClass. Use
`storageClassName: ceph-rbd` in consumer PVCs. Connection secrets and exports
must remain outside Git. The standalone Ceph cluster remains usable without
this Kubernetes integration.

## Validation

```sh
make -C imladris syntax
make -C khazad-dum validate
kustomize build erebor/clusters/smaug
```

These validate local configuration; live VM, Ceph readiness, and PVC I/O checks
are still required. Ceph provisioning follows the
[cephadm bootstrap workflow](https://docs.ceph.com/en/squid/cephadm/install/).
