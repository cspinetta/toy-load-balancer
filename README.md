Toy Load Balancer
=================================
A little example of a Load Balancer on HTTP protocol.

Written in [Rust] language.

## Start server

```bash
cargo run
```

**Some curl's examples:**

```bash
curl -i X POST -H "Content-Type: application/json" -d '{"key1": "value1"}' 'http://localhost:3000/echo'
```

[Rust]:https://www.rust-lang.org/en-US/index.html
