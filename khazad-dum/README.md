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
│   └── khazad-dum/
│       ├── kustomization.yaml    # core plus selected optional labs
│       ├── core.yaml             # composes the reusable platform
│       └── azanulbizar/          # opt-in lab reconciliation catalog
├── infrastructure/
│   └── base/
│       ├── cilium/
│       ├── traefik/
│       ├── cert-manager/
│       ├── traefik-crds/
│       └── metrics/
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
make bootstrap CLUSTER=khazad-dum \
  REPO_URL=https://github.com/joerivrij/middleearth.git \
  BRANCH=main
```

`make bootstrap` installs the Flux controllers, creates the `khazad-dum`
source, and points the cluster at `khazad-dum/clusters/<name>`. The current
default branch is `main`; override `BRANCH` or `KUBECONFIG`
when needed.

For a disposable local test, follow the
[single-node k0s on Lima guide](docs/lima-single-node.md).

After Imladris has prepared the node, the short path is:

```sh
make install
```

This uses Imladris's generated kubeconfig, seeds Cilium so the Flux
controllers can start, and reconciles `clusters/khazad-dum`.

For a production repository, `flux bootstrap github` is also a good option
because it configures deploy credentials. Its sync path should be
`khazad-dum/clusters/<name>`.

## Enter Azanulbizar

Optional labs are catalogued in `clusters/khazad-dum/azanulbizar`. The core
does not include any of them by default. Enable one in the cluster
`kustomization.yaml`:

```yaml
resources:
  - core.yaml
  - azanulbizar/erebor.yaml
```

Use `azanulbizar` to enable the whole catalog. Every entry points at a
stable `cluster/` directory owned by the lab. That directory chooses the lab's
overlay, so changing from Bilbo to Thorin or Wormtongue to Treebeard never
changes Khazad-dûm.

## Configuration policy

Chart versions are constrained to a supported minor line and updates are
deliberate. Values in the shared base must be portable. Hardware-, cloud-, and
cluster-specific settings belong in a cluster overlay or a separate component.
In particular:

- Cilium defaults to Kubernetes IPAM and does not replace kube-proxy.
- Cilium is installed in the dedicated `cilium` namespace during both
  bootstrap seeding and Flux reconciliation.
- The local profile exposes Traefik as NodePort `30080`/`30443`; Imladris
  forwards those to macOS `5687`/`8443`.
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
