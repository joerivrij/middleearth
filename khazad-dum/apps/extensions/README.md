# External lab extensions

These manifests are copyable examples, not part of the platform base and not
enabled by default. Replace each `OWNER` and select the desired profile, then
add the chosen manifest to a cluster definition's `kustomization.yaml`.

- `isengard.yaml` installs ECK and the Wormtongue Elasticsearch profile.
- `morannon.yaml` installs Kyverno with Morannon's audit-only baseline.
- `gondolin.yaml` reconciles a separately keyed SOPS secrets repository.

Each extension reconciles from its own Git source. Adding or removing one does
not modify `infrastructure/base`.
