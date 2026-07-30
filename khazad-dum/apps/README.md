# Applications

This directory contains optional additions composed by cluster definitions.
The platform base never imports them automatically.

`erebor/` is the first consumer: its Rook/Ceph storage and lab-specific Cilium
and Traefik overlays are selected by `clusters/erebor/<environment>`. Future
labs can add sibling directories without changing `infrastructure/base`.

`extensions/` contains opt-in Flux source examples for labs that live in their
own repositories, including Isengard and Morannon.
