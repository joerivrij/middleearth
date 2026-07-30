# Applications

This directory contains optional additions composed by cluster definitions.
The platform base never imports them automatically.

`erebor/` currently contains the Rook/Ceph resources consumed by Erebor. Erebor
owns its Bilbo, Thorin, and Smaug selection below `erebor/cluster`; the
Khazad-dûm core only sees the stable Erebor entrypoint through Azanulbizar.

Other optional labs are also selected through
`clusters/khazad-dum/azanulbizar`, never from the platform base.
