Toy Load Balancer
=================================
A minimalistic Load Balancer on top of HTTP protocol.

Written in [Rust] language.

## Start server

Add settings if it's necessary override the default one: `/config/local.toml`

And execute from the terminal:

```bash
cargo run
```

## Start TEST server

```bash
cargo run --example server-test 0.0.0.0:3001
```

Some curls:

```bash
curl -i 'http://localhost:3001/ping'
curl -i 'http://localhost:3001/medium-payload'
curl -i 'http://localhost:3001/large-payload'
curl -i 'http://localhost:3001/custom-payload?size=100'
curl -i -X POST -d 'some body' 'http://localhost:3001/echo'
```

## Benchmarks
A continuación presentamos un benchmark para comparar resultados entre nuestro load balancer y varios load balancer comerciales. Los load balancers que elegimos son HAProxy y nginx, y para realizar las pruebas utilizamos la herramienta Apache Benchmark. Para las pruebas utilizamos un server que responde a pedidos GET con respuestas de tamaño dinámico en base a un parámetro del request, la idea es tener varias instancias de este server y poder balancearlo. El código de estos servers se encuentra en server-example.js dentro del directorio examples, y además de los resultados de ab en el README se pueden ver otros en la carpeta benches.

Configuraciones del sistema:
```
Linux 4.4.0-83-generic x86_64 x86_64 x86_64 GNU/Linux
Architecture:          x86_64
CPU op-mode(s):        32-bit, 64-bit
Byte Order:            Little Endian
CPU(s):                4
On-line CPU(s) list:   0-3
Thread(s) per core:    2
Core(s) per socket:    2
Socket(s):             1
NUMA node(s):          1
Vendor ID:             GenuineIntel
CPU family:            6
Model:                 58
Model name:            Intel(R) Core(TM) i5-3317U CPU @ 1.70GHz
Stepping:              9
CPU MHz:               1492.679
CPU max MHz:           2600,0000
CPU min MHz:           800,0000
BogoMIPS:              3391.96
Virtualization:        VT-x
L1d cache:             32K
L1i cache:             32K
L2 cache:              256K
L3 cache:              3072K
NUMA node0 CPU(s):     0-3
```

A continuación planteamos tres escenarios distintos:

### Primer escenario
- Cantidad de servidores a balancear: 4
- Tipo de pedidos: GET con respuestas del servidor de 1K
- Request totales: 500
- Request concurrentes: 10

**Toy Load Balancer**
```
ab -n 500 -c 10 -g toy-load-balancer-1K.tsv http://127.0.0.1:3000/1024
```

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
```
ab -n 500 -c 10 -g haproxy-load-balancer-1K.tsv http://127.0.0.1:80/1024
```

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
```
ab -n 500 -c 10 -g nginx-load-balancer-1K.tsv http://127.0.0.1:81/1024
```

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

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-ha-toy-1K.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-nginx-toy-1k.png)

### Segundo escenario
- Cantidad de servidores a balancear: 4
- Tipo de pedidos: GET con respuestas del servidor de 300K
- Request totales: 500
- Request concurrentes: 10

**Toy Load Balancer**
```
ab -n 500 -c 10 -g toy-load-balancer-300K.tsv http://127.0.0.1:3000/307200
```

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
```
ab -n 500 -c 10 -g haproxy-load-balancer-300K.tsv http://127.0.0.1:80/307200
```

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
```
ab -n 500 -c 10 -g nginx-load-balancer-300K.tsv http://127.0.0.1:80/307200
```

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

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-ha-toy-300K.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-nginx-toy-300K.png)


### Tercer escenario
- Cantidad de servidores a balancear: 8
- Tipo de pedidos: GET con respuestas del servidor de 10K
- Request totales: 900
- Request concurrentes: 20

**Toy Load Balancer**
```
ab -n 900 -c 20 -g toy-load-balancer-eight.tsv http://127.0.0.1:3000/10240
```

Resultados:
```
Document Path:          /10240
Document Length:        10279 bytes

Concurrency Level:      20
Time taken for tests:   1.426 seconds
Complete requests:      900
Failed requests:        0
Total transferred:      9371700 bytes
HTML transferred:       9251100 bytes
Requests per second:    631.20 [#/sec] (mean)
Time per request:       31.686 [ms] (mean)
Time per request:       1.584 [ms] (mean, across all concurrent requests)
Transfer rate:          6418.59 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     7   31  14.5     31     117
Waiting:        7   31  14.5     31     117
Total:          7   31  14.6     31     118

Percentage of the requests served within a certain time (ms)
  50%     31
  66%     35
  75%     38
  80%     41
  90%     44
  95%     49
  98%     79
  99%    103
 100%    118 (longest request)
```

**HAProxy**
```
ab -n 900 -c 20 -g ha-load-balancer-eight.tsv http://127.0.0.1:80/10240
```

Resultados:
```
Document Path:          /10240
Document Length:        10254 bytes

Concurrency Level:      20
Time taken for tests:   1.290 seconds
Complete requests:      900
Failed requests:        0
Total transferred:      9319500 bytes
HTML transferred:       9228600 bytes
Requests per second:    697.85 [#/sec] (mean)
Time per request:       28.659 [ms] (mean)
Time per request:       1.433 [ms] (mean, across all concurrent requests)
Transfer rate:          7056.86 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     6   28  10.6     29      62
Waiting:        4   28  10.6     29      62
Total:          6   28  10.6     29      62

Percentage of the requests served within a certain time (ms)
  50%     29
  66%     33
  75%     36
  80%     38
  90%     41
  95%     44
  98%     53
  99%     56
 100%     62 (longest request)
```

**Nginx**
```
ab -n 5000 -c 20 -g nginx-load-balancer-eight.tsv http://127.0.0.1:81/10240
```

Resultados:
```
Document Path:          /10240
Document Length:        10254 bytes

Concurrency Level:      20
Time taken for tests:   1.206 seconds
Complete requests:      900
Failed requests:        0
Total transferred:      9347400 bytes
HTML transferred:       9228600 bytes
Requests per second:    746.37 [#/sec] (mean)
Time per request:       26.796 [ms] (mean)
Time per request:       1.340 [ms] (mean, across all concurrent requests)
Transfer rate:          7570.12 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       1
Processing:     4   26   9.3     28      59
Waiting:        4   26   9.3     28      59
Total:          5   27   9.3     28      59

Percentage of the requests served within a certain time (ms)
  50%     28
  66%     31
  75%     33
  80%     35
  90%     38
  95%     40
  98%     42
  99%     45
 100%     59 (longest request)
 ```
**Toy Load Balancer vs HAProxy**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-ha-toy-eight.png)

**Toy Load Balancer vs Nginx**

![alt text](https://raw.githubusercontent.com/cspinetta/toy-load-balancer/master/benches/comparison-nginx-toy-eight.png)

### Conclusiones
A pesar de que los tiempos para request con respuestas más pequeñas no son tan similares a otros load balancers, si se puede notar que para request GET con respuestas más extensas nuestro load balancer se asemeja bastante a otros load balancers del mercado.

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
