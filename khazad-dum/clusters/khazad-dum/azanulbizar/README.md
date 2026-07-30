# Azanulbizar

Azanulbizar is Khazad-dûm's catalog of optional Middle-earth labs. The core
cluster does not include any entry by default.

Each manifest reconciles a stable `cluster/` entrypoint owned by that lab.
The lab chooses its own overlay there, so Khazad-dûm never needs to know names
such as Bilbo, Thorin, Smaug, Wormtongue, Treebeard, or Saruman.

Enable one lab by adding it to the parent `kustomization.yaml`:

```yaml
resources:
  - core.yaml
  - azanulbizar/erebor.yaml
```

Enable every catalogued lab with:

```yaml
resources:
  - core.yaml
  - azanulbizar
```
