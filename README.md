Toy Load Balancer
=================================
A little example of a Load Balancer on HTTP protocol.

Written in [Rust] language.

## Start server

```bash
cargo run
```

**Some tests:**

For now, a simple way to test the app is [start a server with python](https://docs.python.org/3/library/http.server.html): `python3 -m http.server 9290` , then start the app and do some requests:

```bash
curl -i 'http://localhost:3000/'

curl -i 'http://localhost:3000/path/to/files'
```

[Rust]:https://www.rust-lang.org/en-US/index.html
