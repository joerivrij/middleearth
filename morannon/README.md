# Morannon

Morannon is the admission-control and policy laboratory for Middle-earth. It is
a Flux extension designed to be attached to a Khazad-dûm-managed cluster.

The first deployment installs Kyverno and three introductory policies:

- discourage container images using `latest` or no tag
- detect privileged containers
- require the recommended name and managed-by labels on controllers

All policies begin with `validationFailureAction: Audit`. They produce policy
reports without blocking workloads. Move a policy to `Enforce` only after its
reports have been reviewed and the affected namespaces are understood.

## Attach to Khazad-dûm

Create a Flux `GitRepository` named `morannon`, then point its root
`Kustomization` at:

```text
./clusters/default
```

An example source and reconciliation pair is provided in
`../khazad-dum/apps/extensions/morannon.yaml`.

## Validate

```sh
make validate
```

