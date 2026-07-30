# Amon Sûl

Amon Sûl is a hands-on BGP learning lab. Grey Havens follows packets; Amon Sûl
follows routes and route advertisements.

The first milestone builds one eBGP session between two Raspberry Pis or
Debian-family Linux hosts on the same LAN:

```text
10.0.0.11, AS 65001                         10.0.0.12, AS 65002
┌──────────────────────┐    eBGP/TCP 179    ┌──────────────────────┐
│ node-a               │◄──────────────────►│ node-b               │
│ dummy: 10.200.1.1/32 │                    │ dummy: 10.200.2.1/32 │
└──────────────────────┘                    └──────────────────────┘
```

Each node originates its dummy-interface `/32`. FRRouting exchanges the
prefixes and Zebra installs the learned route into Linux's routing table.
Stopping one dummy interface removes its connected route, causing BGP to
withdraw the prefix. Starting it again re-advertises it.

## Requirements

- Two Debian, Ubuntu, or Raspberry Pi OS hosts on the same LAN
- Python 3 and SSH access on both hosts
- Ansible Core 2.17+ on the control machine
- Unique LAN addresses and private ASNs
- TCP port 179 permitted between the hosts

There is intentionally no Kubernetes, OVN, container runtime, route reflector,
or UI.

## Configure the lab

```sh
cp -R ansible/inventories/example ansible/inventories/home
$EDITOR ansible/inventories/home/hosts.yml
make setup INVENTORY=ansible/inventories/home/hosts.yml
make verify INVENTORY=ansible/inventories/home/hosts.yml
```

The example uses documentation-only LAN addresses. Replace `ansible_host`,
`bgp_router_id`, and `bgp_neighbor_ip` with the real addresses of the two
hosts. Keep each ASN and advertised prefix unique.

## Withdrawal experiment

```sh
make withdraw INVENTORY=ansible/inventories/home/hosts.yml
make verify-withdrawal INVENTORY=ansible/inventories/home/hosts.yml
make advertise INVENTORY=ansible/inventories/home/hosts.yml
make verify INVENTORY=ansible/inventories/home/hosts.yml
```

`withdraw_target` identifies the node whose dummy prefix is toggled.
`withdraw_observers` identifies the peer where disappearance is checked. Swap
the inventory group membership to run the experiment in the other direction.

Watch the control plane directly:

```sh
ssh node-b sudo vtysh -c 'show bgp ipv4 unicast summary'
ssh node-b sudo vtysh -c 'show bgp ipv4 unicast'
ssh node-b ip route
```

See [docs/bgp-walkthrough.md](docs/bgp-walkthrough.md) for what each table
means and how best-path selection relates to this deliberately simple lab.

## Reset

```sh
make reset INVENTORY=ansible/inventories/home/hosts.yml
```

Reset removes the managed FRR configuration and dummy-interface unit, then
stops FRR. Packages remain installed by default. To remove them too:

```sh
make reset INVENTORY=ansible/inventories/home/hosts.yml \
  EXTRA_ARGS='-e amonsul_reset_remove_packages=true'
```

## Layout

```text
amonsul/
├── ansible/
│   ├── inventories/example/
│   ├── playbooks/
│   └── roles/bgp_node/
├── docs/
├── scripts/
├── Makefile
└── README.md
```

