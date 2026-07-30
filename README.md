# Middleearth

One repository to rule the homelab.

`middleearth` is the container for the foundations and focused experiments used
to build, rebuild, and understand the homelab. Each realm owns one concern and
can consume the shared provisioning and Kubernetes foundations without folding
its implementation into them.

## The Map

```text
middleearth/
├── bree/
│   └── workstation bootstrap
├── imladris/
│   └── Ansible and machine provisioning
├── khazad-dum/
│   └── Kubernetes platform
├── eregion/
│   └── virtualization: libvirt, cloud-init, vsock, QEMU, images
├── erebor/
│   └── Ceph and storage
├── grey-havens/
│   └── networking and OVN
├── amonsul/
│   └── BGP and routing
├── gondolin/
│   └── secrets
├── morannon/
│   └── admission control and policy
├── isengard/
│   └── Elasticsearch, search, and observability
├── LICENSE
└── README.md
```

## Project boundaries

| Project | Responsibility |
| --- | --- |
| [Bree](bree/README.md) | Bootstrap the operator's workstation. |
| [Imladris](imladris/README.md) | Provision Linux machines with reusable Ansible roles. |
| [Khazad-dûm](khazad-dum/README.md) | Provide the reusable Kubernetes and Flux platform. |
| [Eregion](eregion/README.md) | Explore libvirt, QEMU, cloud-init, vsock, and VM images. |
| [Erebor](erebor/README.md) | Explore Ceph and storage. |
| [Grey Havens](grey-havens/README.md) | Explore networking, packet flow, OVS, and OVN. |
| [Amon Sûl](amonsul/README.md) | Explore BGP, route exchange, and routing decisions. |
| [Gondolin](gondolin/README.md) | Explore secrets management and rotation. |
| [Morannon](morannon/README.md) | Explore admission control and policy enforcement. |
| [Isengard](isengard/README.md) | Explore Elasticsearch, search, and observability. |

Imladris and Khazad-dûm are the reusable foundations. The other realms are
experiments that may add inventories, machine profiles, platform additions, or
applications without changing those foundations' core behavior.

## License

MIT. See [LICENSE](LICENSE).
