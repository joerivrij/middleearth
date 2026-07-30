# Erebor add-on overlays

```text
apps/erebor/overlays/
├── bilbo
├── thorin
└── smaug
```

## Bilbo — “An Unexpected Journey”

Minimal development cluster.

## Thorin — “The King Under the Mountain”

Standard homelab / HA deployment.

## Smaug — “The Dragon’s Hoard”

Large-scale storage and performance testing.

The names also map surprisingly well to what the environments represent:

| Overlay | Story | Infrastructure analogy |
| --- | --- | --- |
| `bilbo` | The unexpected journey begins. | Small, single-node experiment, proving the concept. |
| `thorin` | Reclaiming Erebor and building a kingdom. | A realistic HA cluster for day-to-day development and testing. |
| `smaug` | The dragon sitting on an enormous hoard. | The largest cluster with lots of storage and resources. |

The previous size names map as follows:

| Overlay | Size |
| --- | --- |
| `bilbo` | Small |
| `thorin` | Medium |
| `smaug` | Large |

Each environment is split into component-level Kustomize overlays. The Flux
Kustomizations in `erebor/clusters/` reference those components separately
instead of applying an entire environment in one reconciliation.
