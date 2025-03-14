Example showing dualstack TCP socket via `socket2` and `tokio`.

```
➜  tokio-dual git:(master) ✗ cargo run

Server listening on [::]:36943 (dual-stack).
Connected via IPv6!
Sent 'ping' (IPv6)
Accepted connection from [::1]:53690
Server received 'ping' -> sending 'pong'.
IPv6 client received: "pong"
Accepted connection from [::ffff:127.0.0.1]:55924
Connected via IPv4!
Sent 'ping' (IPv4)
Server received 'ping' -> sending 'pong'.
IPv4 client received: "pong"
Done.
```