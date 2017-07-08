Toy Load Balancer
=================================
A little example of a Load Balancer on HTTP protocol.

Written in [Rust] language.

## Start server

```bash
cargo run
```

## Benchmarks
A continuación presentamos un benchmark para comparar resultados entre nuestro load balancer y varios load balancer comerciales. Los load balancers que elegimos son HAProxy y nginx, y para realizar las pruebas utilizamos la herramienta Apache Benchmark. Para las pruebas utilizamos un server que responde a pedidos GET con respuestas de tamaño dinámico en base a un parámetro del request, la idea es tener varias instancias de este server y poder balancearlo. El código de estos servers se encuentra en server-example.js dentro del directorio examples. A continuación planteamos tres escenarios distintos:

### Primer escenario
- Cantidad de servidores a balancear: 4
- Tipo de pedidos: GET con respuestas del servidor de 1K
- Request totales: 500
- Request concurrentes: 10

**Toy Load Balancer**

ab -n 500 -c 10 -g toy-load-balancer-1K.tsv http://127.0.0.1:3000/1024

Resultados:
```
Document Path:          /1024
Document Length:        1055 bytes

Concurrency Level:      10
Time taken for tests:   0.749 seconds
Complete requests:      500
Failed requests:        0
Total transferred:      594500 bytes
HTML transferred:       527500 bytes
Requests per second:    667.96 [#/sec] (mean)
Time per request:       14.971 [ms] (mean)
Time per request:       1.497 [ms] (mean, across all concurrent requests)
Transfer rate:          775.59 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     2   15   9.9     12      52
Waiting:        2   13   8.8     10      50
Total:          2   15   9.9     12      53

Percentage of the requests served within a certain time (ms)
  50%     12
  66%     18
  75%     21
  80%     23
  90%     29
  95%     33
  98%     41
  99%     48
 100%     53 (longest request)
```

**HAProxy**

ab -n 500 -c 10 -g haproxy-load-balancer-1K.tsv http://127.0.0.1:80/1024

Resultados:
```
Document Path:          /1024
Document Length:        1038 bytes

Concurrency Level:      10
Time taken for tests:   0.256 seconds
Complete requests:      500
Failed requests:        0
Total transferred:      569500 bytes
HTML transferred:       519000 bytes
Requests per second:    1951.52 [#/sec] (mean)
Time per request:       5.124 [ms] (mean)
Time per request:       0.512 [ms] (mean, across all concurrent requests)
Transfer rate:          2170.69 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     1    5   2.8      4      16
Waiting:        1    5   2.7      4      16
Total:          2    5   2.8      4      16

Percentage of the requests served within a certain time (ms)
  50%      4
  66%      5
  75%      6
  80%      6
  90%     10
  95%     11
  98%     14
  99%     15
 100%     16 (longest request)
```

**Nginx**

ab -n 500 -c 10 -g nginx-load-balancer-1K.tsv http://127.0.0.1:81/1024

Resultados:
```
Document Path:          /1024
Document Length:        1038 bytes

Concurrency Level:      10
Time taken for tests:   0.228 seconds
Complete requests:      500
Failed requests:        0
Total transferred:      585000 bytes
HTML transferred:       519000 bytes
Requests per second:    2193.27 [#/sec] (mean)
Time per request:       4.559 [ms] (mean)
Time per request:       0.456 [ms] (mean, across all concurrent requests)
Transfer rate:          2505.98 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     1    4   2.1      4      14
Waiting:        1    4   2.1      4      14
Total:          1    4   2.1      4      14

Percentage of the requests served within a certain time (ms)
  50%      4
  66%      5
  75%      6
  80%      6
  90%      8
  95%      9
  98%     10
  99%     12
 100%     14 (longest request)
 ```
**Toy Load Balancer vs HAProxy**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/comparison-ha-toy-1K.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/comparison-nginx-toy-1k.png)

### Segundo escenario
- Cantidad de servidores a balancear: 4
- Tipo de pedidos: GET con respuestas del servidor de 300K
- Request totales: 500
- Request concurrentes: 10

**Toy Load Balancer**

ab -n 500 -c 10 -g toy-load-balancer-300K.tsv http://127.0.0.1:3000/307200

Resultados:
```
Document Path:          /307200
Document Length:        307534 bytes

Concurrency Level:      10
Time taken for tests:   17.758 seconds
Complete requests:      500
Failed requests:        491
   (Connect: 0, Receive: 0, Length: 491, Exceptions: 0)
Total transferred:      153831054 bytes
HTML transferred:       153764054 bytes
Requests per second:    28.16 [#/sec] (mean)
Time per request:       355.167 [ms] (mean)
Time per request:       35.517 [ms] (mean, across all concurrent requests)
Transfer rate:          8459.43 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:   131  352 103.6    359     579
Waiting:      120  346 103.3    352     574
Total:        131  352 103.6    359     579

Percentage of the requests served within a certain time (ms)
  50%    359
  66%    400
  75%    427
  80%    446
  90%    479
  95%    495
  98%    529
  99%    549
 100%    579 (longest request)
```

**HAProxy**

ab -n 500 -c 10 -g haproxy-load-balancer-300K.tsv http://127.0.0.1:80/307200

Resultados:
```
Document Path:          /307200
Document Length:        307214 bytes

Concurrency Level:      10
Time taken for tests:   15.203 seconds
Complete requests:      500
Failed requests:        0
Total transferred:      153657500 bytes
HTML transferred:       153607000 bytes
Requests per second:    32.89 [#/sec] (mean)
Time per request:       304.059 [ms] (mean)
Time per request:       30.406 [ms] (mean, across all concurrent requests)
Transfer rate:          9870.20 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       2
Processing:   106  301 100.7    322     503
Waiting:      105  301 100.7    322     503
Total:        107  301 100.7    322     503

Percentage of the requests served within a certain time (ms)
  50%    322
  66%    356
  75%    361
  80%    411
  90%    443
  95%    449
  98%    455
  99%    480
 100%    503 (longest request)
```

**Nginx**

ab -n 500 -c 10 -g nginx-load-balancer-300K.tsv http://127.0.0.1:80/307200

Resultados:
```
Document Path:          /307200
Document Length:        307214 bytes

Concurrency Level:      10
Time taken for tests:   16.186 seconds
Complete requests:      500
Failed requests:        0
Total transferred:      153657500 bytes
HTML transferred:       153607000 bytes
Requests per second:    30.89 [#/sec] (mean)
Time per request:       323.725 [ms] (mean)
Time per request:       32.372 [ms] (mean, across all concurrent requests)
Transfer rate:          9270.61 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.0      0       1
Processing:    86  321 109.8    335     542
Waiting:       85  320 109.8    334     541
Total:         86  321 109.7    335     542

Percentage of the requests served within a certain time (ms)
  50%    335
  66%    373
  75%    407
  80%    421
  90%    466
  95%    488
  98%    511
  99%    518
 100%    542 (longest request)
 ```
**Toy Load Balancer vs HAProxy**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/comparison-ha-toy-300K.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/comparison-nginx-toy-300K.png)


### Tercer escenario
- Cantidad de servidores a balancear: 8
- Tipo de pedidos: GET con respuestas del servidor de 10K
- Request totales: 5000
- Request concurrentes: 20

**Toy Load Balancer**

ab -n 5000 -c 20 -g toy-load-balancer-eight.tsv http://127.0.0.1:3000/10240

Resultados:
```

```

**HAProxy**

ab -n 5000 -c 20 -g ha-load-balancer-eight.tsv http://127.0.0.1:80/10240

Resultados:
```
Document Path:          /10240
Document Length:        10254 bytes

Concurrency Level:      20
Time taken for tests:   6.089 seconds
Complete requests:      5000
Failed requests:        0
Total transferred:      51775000 bytes
HTML transferred:       51270000 bytes
Requests per second:    821.22 [#/sec] (mean)
Time per request:       24.354 [ms] (mean)
Time per request:       1.218 [ms] (mean, across all concurrent requests)
Transfer rate:          8304.42 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       4
Processing:     3   24   8.5     25      65
Waiting:        3   24   8.5     25      65
Total:          5   24   8.5     25      65

Percentage of the requests served within a certain time (ms)
  50%     25
  66%     28
  75%     30
  80%     32
  90%     35
  95%     36
  98%     40
  99%     43
 100%     65 (longest request)
```

**Nginx**

ab -n 5000 -c 20 -g nginx-load-balancer-eight.tsv http://127.0.0.1:81/10240

Resultados:
```
Document Path:          /10240
Document Length:        10254 bytes

Concurrency Level:      20
Time taken for tests:   5.717 seconds
Complete requests:      5000
Failed requests:        0
Total transferred:      51930000 bytes
HTML transferred:       51270000 bytes
Requests per second:    874.51 [#/sec] (mean)
Time per request:       22.870 [ms] (mean)
Time per request:       1.143 [ms] (mean, across all concurrent requests)
Transfer rate:          8869.81 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       2
Processing:     4   23   8.3     24      70
Waiting:        4   23   8.3     24      70
Total:          4   23   8.3     24      71

Percentage of the requests served within a certain time (ms)
  50%     24
  66%     26
  75%     28
  80%     30
  90%     33
  95%     34
  98%     36
  99%     40
 100%     71 (longest request)
 ```
**Toy Load Balancer vs HAProxy**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/resources/comparison-ha-toy-eight.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/resources/comparison-nginx-toy-eight.png)

## Design decisions and selected technology
Sigamos en español para facilitar la evaluación =)

**Por la naturaleza del caso de uso, los aspectos que más valoramos son:**
- Aceptar un throughput alto.
- Mínimo overhead entre el cliente y el host final.
- Capacidad para escalar en volumen de carga.
- Bajo footprint.
- Idealmente uso de sockets non-blocking (dado que el problema es IO bound).


### Tecnología usada:

Valoramos varias estrategias y tecnologías: actores con Elixir, Ruby con Puma, Go, Rust, ~~C++~~.

**Finalmente elegimos [Rust]. Los motivos fueron:**

- Es de bajo nivel (se acerca bastante a C), lo cual permite ~~reducir~~ manejar mejor las alocaciones en memoria, y a pesar de eso, provee muchas validaciones desde el compilador y muchos features de lenguajes de alto nivel.
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

Implementamos una arquitectura basada en un _grupo de event loops_ (por default la misma cantidad de core de la máquina host), y cada event loop implementa la misma lógica: escuchar conexiones TCP en un puerto, resolver el protocolo HTTP, decidir a que host redireccionar y hacer la conexión con el host final, para luego propagar la respuesta de este al cliente inicial. La solución está pensada para correr sobre un sistema _Unix_ y _kernel >= 3.9_, para aprovechar la opción [SO_REUSEPORT] que permite abrir N sockets asociados al mismo puerto, de esta manera tenemos N event loops, todos escuchando el mismo puerto.

## License

MIT

[Ownership]:https://doc.rust-lang.org/book/second-edition/ch04-00-understanding-ownership.html
[Lifetime]:https://doc.rust-lang.org/book/second-edition/ch10-03-lifetime-syntax.html
[MIO]:https://github.com/carllerche/mio
[Tokio]:https://github.com/tokio-rs/tokio-core
[Rust]:https://www.rust-lang.org/en-US/index.html
[SO_REUSEPORT]:https://lwn.net/Articles/542629/
