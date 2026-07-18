# Grey Havens

Grey Havens is a small "life of a packet" laboratory. Give it an HTTP or HTTPS
URL and it shows the observable stages of the request: URL parsing, DNS, TCP,
TLS, connection selection, HTTP headers, first response byte, and response body.
Each stage explains what is happening and exposes details such as IP addresses,
local and remote sockets, TLS cipher and certificate, protocol negotiation,
response metadata, and timing. The same events are logged in the terminal so
the browser view and the service output can be compared.

The page also introduces all seven OSI layers using one HTTP request as an
example. Every trace event is labelled with its relevant OSI layer. Grey Havens
is explicit about its current boundary: Go directly observes the transport,
presentation, and application activity, while inspecting IP packets, local
frames, and physical transmission will require a future packet-capture view.

## Run it

Go 1.22 or newer is required.

```sh
cd grey-havens
go run .
```

Then open <http://127.0.0.1:8080>. The service only listens on localhost by
default. To choose a different address:

```sh
GREY_HAVENS_ADDR=:8080 go run .
```

The tracer follows one request only; redirects are returned as `3xx` responses
so each voyage stays easy to understand. Response reads are capped at 1 MiB and
requests time out after 15 seconds.

## Where this could sail next

- draw the browser, router, DNS resolver, and server as an animated route
- explain headers and protocol differences in plain language
- add ICMP/traceroute hops where the host permits raw sockets
- capture traffic with `tcpdump` or eBPF and correlate packets with the timeline
- provide deliberately slow and broken local endpoints for experiments
