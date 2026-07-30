# Single-node k0s on Lima

This walkthrough creates one Debian Lima VM, installs a single-node k0s
controller with Imladris, seeds Cilium, and then installs Flux and the
Khazad-dûm platform.

It uses the generic `clusters/example` platform definition. It deliberately
does not use `clusters/erebor/bilbo`, because Bilbo also installs the
Erebor-specific Traefik configuration and Rook/Ceph.

## Prerequisites

Run this on macOS with:

- Homebrew
- Lima (`brew install lima`)
- Ansible Core 2.17 or newer
- `kubectl`
- the Flux CLI
- the Cilium CLI

For example:

```sh
brew install lima ansible kubectl fluxcd/tap/flux cilium-cli
```

The Khazad-dûm manifests must be committed and pushed to the `middleearth`
repository. Flux clones that repository and reconciles paths below
`./khazad-dum`. Flux cannot reconcile uncommitted files from the local
checkout.

The commands below are run from the `middleearth` checkout.

## 1. Create the VM and install k0s

```sh
cd imladris
make single-node
```

If the VM was created by an older version of this configuration and Ansible
reports `sudo: a password is required`, recreate that disposable VM once so
Lima can apply the system provisioning:

```sh
limactl stop imladris
limactl delete imladris
make single-node
```

The former test VM was named `khazad-dum`. Remove that old instance before
creating `imladris`, because both would otherwise compete for host port 6443:

```sh
limactl stop khazad-dum
limactl delete khazad-dum
```

This creates a Lima VM named `imladris`, installs the single-node k0s cluster
and Kubernetes node named `khazad-dum`, waits for its API, and writes:

```text
imladris/machine-build/khazad-dum.kubeconfig
```

Select it for the remaining commands:

```sh
export KUBECONFIG="$PWD/machine-build/khazad-dum.kubeconfig"
kubectl get nodes
```

The node will initially be `NotReady`. That is expected because k0s was
created with `network.provider: custom` and Cilium has not been installed yet.

## 2. Install Khazad-dûm

Flux controllers are normal pods and cannot start without a functioning CNI.
The install target first seeds the same Cilium minor version that Khazad-dûm
manages, then installs Flux and reconciles `clusters/example`:

```sh
cd ../khazad-dum
make install
```

Check the cluster:

```sh
export KUBECONFIG="../imladris/machine-build/khazad-dum.kubeconfig"
kubectl wait node --all --for=condition=Ready --timeout=5m
cilium status --wait
kubectl get nodes
kubectl get pods --all-namespaces
```

Override `REPO_URL` or `BRANCH` when testing another fork or branch:

```sh
make install \
  REPO_URL=https://github.com/example/middleearth.git \
  BRANCH=my-branch
```

## 3. Follow reconciliation

```sh
flux get sources git
flux get kustomizations
flux get helmreleases --all-namespaces
flux logs --follow
```

The expected dependency order is:

```text
cilium
├── cert-manager
├── traefik-crds
│   └── traefik
└── metrics
```

Useful final checks:

```sh
kubectl get pods --all-namespaces
kubectl get helmreleases --all-namespaces
kubectl top nodes
```

The Traefik Service may remain without an external address. Lima has no
bare-metal load-balancer implementation by default; this does not prevent the
platform components from becoming ready.

## Reconcile after a change

Commit and push the change, then either wait for the interval or run:

```sh
flux reconcile source git khazad-dum
flux reconcile kustomization khazad-dum --with-source
```

## Reset the lab

Delete only this explicitly named Lima VM:

```sh
limactl stop imladris
limactl delete imladris
```

The kubeconfig under `imladris/machine-build/` can then be removed manually.
