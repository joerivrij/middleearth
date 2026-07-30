# Following a route through Amon Sûl

## 1. The local route

The `amonsul-dummy.service` creates a dummy interface and assigns a `/32`.
Linux installs that address as a connected route. FRR's `network` statement
does not invent reachability: `bgp network import-check` requires the prefix to
exist in the local routing table before BGP originates it.

```sh
ip address show amonsul0
ip route show table local
sudo vtysh -c 'show bgp ipv4 unicast'
```

## 2. The eBGP session

The nodes connect directly over their LAN addresses. Because their ASNs differ,
the session is external BGP. BGP uses TCP port 179. The OPEN messages establish
identity and capabilities; KEEPALIVEs maintain the session.

```sh
sudo vtysh -c 'show bgp ipv4 unicast summary'
sudo vtysh -c 'show bgp neighbors'
```

`Established` means the control-plane session is ready. It does not by itself
prove that a prefix was advertised, selected, or installed.

## 3. Advertisement, selection, and installation

The peer receives an UPDATE containing the prefix and an AS_PATH with the
originating ASN. With only one path, selection is intentionally unsurprising:
the received path is valid and becomes best. Zebra then installs it into the
Linux kernel through the peer's LAN address.

```sh
sudo vtysh -c 'show bgp ipv4 unicast 10.200.1.1/32'
ip route show 10.200.1.1/32
```

The BGP table is the protocol's view; `ip route` is the forwarding plane's
view. A route can be received without being best, and a best route can fail to
install, so inspect both.

## 4. Withdrawal and re-advertisement

Stopping the dummy service deletes the interface and connected route. Import
checking makes the local BGP path ineligible, so FRR sends a withdrawal. The
peer removes the path and Zebra removes the kernel route.

Starting the service restores the connected route. FRR originates a new UPDATE,
the peer selects it again, and Zebra reinstalls it.

```sh
make withdraw
make verify-withdrawal
make advertise
make verify
```

The eBGP session stays Established throughout. This separates loss of a prefix
from loss of a neighbor and makes the UPDATE/withdrawal behavior easy to see.

## Next experiments

After the first milestone is comfortable:

1. Advertise the same prefix from both nodes and compare path attributes.
2. Add a third plain Linux router and observe AS_PATH loop prevention.
3. Change local preference or AS-path prepending and predict the best path.
4. Capture TCP port 179 with `tcpdump` and correlate UPDATEs with the tables.

These are follow-on exercises, not part of the initial automation.

