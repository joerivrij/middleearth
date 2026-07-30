# Khazad-dûm

Khazad-dûm is the reusable Kubernetes foundation for the Middle-earth labs. It
owns the platform; Erebor, Grey Havens, and future realms own their
applications.

Flux reconciles an opinionated baseline:

- Cilium for cluster networking
- Traefik for ingress
- cert-manager for certificate automation
- metrics-server for the Kubernetes resource metrics API

ExternalDNS and the Prometheus/Grafana stack are intentionally deferred until
their credentials, DNS provider, storage, and retention requirements are known.

## Repository map

```text
khazad-dum/
├── apps/                         # optional, platform-owned applications
│   └── erebor/                   # optional Erebor storage and overlays
├── clusters/
│   └── example/
│       ├── kustomization.yaml    # cluster reconciliation entrypoint
│       └── platform.yaml         # composes the reusable platform
├── infrastructure/
│   └── base/
│       ├── cilium/
│       ├── traefik/
│       ├── cert-manager/
│       └── metrics-server/
└── Makefile
```

Like Erebor's GitOps layout, every component lives below `infrastructure/base`
and is independently reconcilable. Cluster definitions compose them with Flux
`Kustomization` resources and dependencies; they do not copy or patch the
shared base.

## Bootstrap a cluster

The target cluster must be reachable and created without another CNI if Cilium
is to be its primary CNI. Install `flux` and `kubectl`, then:

```sh
cp -R clusters/example clusters/my-cluster
make bootstrap CLUSTER=my-cluster \
  REPO_URL=https://github.com/joerivrij/middleearth.git \
  BRANCH=feat/add-experiments
```

`make bootstrap` installs the Flux controllers, creates the `khazad-dum`
source, and points the cluster at `khazad-dum/clusters/<name>`. The current
default branch is `feat/add-experiments`; override `BRANCH` or `KUBECONFIG`
when needed.

For a disposable local test, follow the
[single-node k0s on Lima guide](docs/lima-single-node.md).

After Imladris has prepared the node, the short path is:

```sh
make install
```

This uses Imladris's generated kubeconfig, seeds Cilium so the Flux
controllers can start, and reconciles `clusters/example`.

Erebor uses k0s with a custom CNI, so Cilium must exist before Flux controllers
can become ready. Its convenience target seeds the same Cilium minor version
managed by the base, then bootstraps the selected Erebor definition:

```sh
make bootstrap-erebor ENV=bilbo \
  KUBECONFIG=../erebor/machine/kubeconfig \
  REPO_URL=https://github.com/joerivrij/middleearth.git \
  BRANCH=feat/add-experiments
```

For a production repository, `flux bootstrap github` is also a good option
because it configures deploy credentials. Its sync path should be
`khazad-dum/clusters/<name>`.

## Add a lab

A lab remains in its own repository. Add two manifests to the relevant cluster
directory and list them in that directory's `kustomization.yaml`:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: GitRepository
metadata:
  name: erebor
  namespace: flux-system
spec:
  interval: 10m
  ref:
    branch: main
  url: https://github.com/OWNER/erebor.git
---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: erebor
  namespace: flux-system
spec:
  dependsOn:
    - name: platform-certificates
    - name: platform-ingress
  interval: 10m
  path: ./clusters/production
  prune: true
  sourceRef:
    kind: GitRepository
    name: erebor
  wait: true
```

That is the extension boundary: adding a lab changes only a cluster definition,
never `infrastructure/`.

## Configuration policy

Chart versions are constrained to a supported minor line and updates are
deliberate. Values in the shared base must be portable. Hardware-, cloud-, and
cluster-specific settings belong in a cluster overlay or a separate component.
In particular:

- Cilium defaults to Kubernetes IPAM and does not replace kube-proxy.
- Traefik uses a `LoadBalancer` Service; bare-metal clusters need an address
  provider such as Cilium L2 announcements or MetalLB.
- No default `ClusterIssuer` is included because ACME email, challenge type,
  and DNS credentials are environment-specific.
- metrics-server uses normal kubelet TLS verification. Do not add
  `--kubelet-insecure-tls` to the shared base.

## Validate

```sh
make validate
make status
```

`validate` renders every Kustomize entrypoint locally. `status` queries Flux in
the selected cluster.
