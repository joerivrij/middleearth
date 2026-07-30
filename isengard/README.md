# Isengard

Isengard is the Elasticsearch, search, and observability laboratory for
Middle-earth. It is a Flux extension designed to be attached to a
Khazad-dûm-managed cluster.

The initial deployment follows the ECK pattern from Odysseia-Greek's
Themistokles: the Elastic operator is managed by a Flux `HelmRelease`, while
Elasticsearch and Kibana are ECK custom resources.

## Profiles

| Profile | Shape | Intended use |
| --- | --- | --- |
| `wormtongue` | One combined node, 1 GiB heap container limit, 10 GiB volume | Minimal learning and API experiments |
| `treebeard` | Three combined nodes, 2 GiB each, 20 GiB volumes | Resilience and shard-placement experiments |
| `saruman` | Three masters, three data/ingest nodes, two Kibana replicas | Role separation and larger observability experiments |

The overlays are cumulative: Treebeard extends Wormtongue, and Saruman extends
Treebeard. Resource and storage requests are intentionally visible and should
be reviewed before selecting a profile.

## Attach to Khazad-dûm

Create a Flux `GitRepository` named `isengard`, then point a root
`Kustomization` at one of:

```text
./clusters/wormtongue
./clusters/treebeard
./clusters/saruman
```

An example source and reconciliation pair is provided in
`../khazad-dum/apps/extensions/isengard.yaml`.

ECK generates the Elasticsearch and Kibana credentials and TLS secrets. No
passwords are stored in Git.

## Validate

```sh
make validate
```

