Toy Load Balancer
=================================
A little example of a Load Balancer on HTTP protocol.

Written in [Rust] language.

## Start server

```bash
cargo run
```

## Some benchmarks

For now, a simple way to test the app is [start a server with python](https://docs.python.org/3/library/http.server.html): `python3 -m http.server 9290` , then start the app and do some requests:

```bash
curl -i 'http://localhost:3000/'

curl -i 'http://localhost:3000/path/to/files'
```

## Design decisions and selected technology
Sigamos en español para facilitar la evaluación =)

**Por la naturaleza del caso de uso, los aspectos que más valoramos son:**
- Aceptar un throughput alto.
- Mínimo overhead entre el cliente y el host final.
- Capacidad para escalar en volumen de carga.
- Bajo footprint.
- Idealmente uso de sockets non-blocking (dado qeu el problema es IO bound).


### Tecnología usada:

Valoramos varias estrategias y tecnologías: actores con Elixir, Ruby con Puma, Go, Rust, ~~C++~~.

**Finalmente elegimos [Rust]. Los motivos fueron:**

- Es de bastante bajo nivel (se acerca bastante a C), pero con muchas validaciones por el compilador y muchos features de lenguajes de alto nivel.
- No usa GC, sino que el compilador introduce el pedido y liberación de memoria, a través de un concepto que llamaron [Ownership].
- Hay librerias para IO bastante maduras: [MIO] para manejo de primitivas non-blocking y [Tokio] para manejo de conexiones.

**Algunos extras que reforzaron la elección:**
- Tipado estático.
- No admite `null` (en reemplazo ofrece el tipo `Option<A>`).
- Introduce Tipos de Datos Algebráicos (tomando estructuras de Lisp y Haskell principalmente).
- Validaciones en tiempo de compilación para [evitar algunos problemas de concurrencia](https://doc.rust-lang.org/book/second-edition/ch16-00-concurrency.html) (introduciendo `Send` y `Sync`, entre otros).
- Prevención de errores clásicos en lenguajes con manejo explícito de punteros, como _dangling references_ o _race data_, en tiempo de compilación a través de [Lifetime].
- Comunidad muy activa (muchos posts, mucho movimiento en StackOverflow, en gitter y por IRC).

**Algunos contras que encontramos (siendo que ninguno tiene experiencia con el lenguaje):**
- Curva de aprendizaje bastante dura.
- Aún no hay un buen soporte para el IDE.
- En general las librerias están muy verdes.

### Arquitectura implementada

Implementamos una arquitectura basada en un _grupo de event loops_ (por default la misma cantidad de core de la máquina host), y cada event loop implementa la misma lógica: escuchar conexiones TCP en un puerto, resolver el protocolo HTTP, decidir a que host redireccionar y hacer la conexión con el host final, para luego propagar la respuesta de este al cliente inicial. La solución está pensada para correr sobre un sistema _Unix_ y kernel >= 3.9, para aprovechar la opción [SO_REUSEPORT] que permite abrir N sockets asociados al mismo puerto, de esta manera tenemos N event loops, todos escuchando el mismo puerto.

## License

MIT

[Ownership]:https://doc.rust-lang.org/book/second-edition/ch04-00-understanding-ownership.html
[Lifetime]:https://doc.rust-lang.org/book/second-edition/ch10-03-lifetime-syntax.html
[MIO]:https://github.com/carllerche/mio
[Tokio]:https://github.com/tokio-rs/tokio-core
[Rust]:https://www.rust-lang.org/en-US/index.html
[SO_REUSEPORT]:https://lwn.net/Articles/542629/
