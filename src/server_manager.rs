extern crate rand;

use duplex;
use std::io::{self, Write};
use std::sync::mpsc::{TryRecvError};
use server_manager::rand::Rng;

pub fn server_manager(channel_vector : Vec<duplex::DuplexStream>) {

    //Falta levantar la configuracion
    //let estrategia = "RR"; //RoundRobin
    let estrategia = "RN";//Random
    let mut ultimo_elegido = 0;
    let servidores_habilitados = vec!["http://192.0.0.1", "http://192.0.0.2", "http://192.0.0.3", "http://192.0.0.4"];

    loop {

        for i in 0..channel_vector.len()
        {
            let value = channel_vector[i].rx.try_recv();
            match value {
                Ok(id) => {
                    println!("Valor: {}", id);

                    //estrategia de asignacion de servers
                    match estrategia
                    {
                       "RR"=>{

                           println! ("{:?}", ultimo_elegido);
                           channel_vector[i].tx.send(servidores_habilitados[ultimo_elegido].to_string());

                           if (ultimo_elegido >= servidores_habilitados.len() - 1)
                           {
                               ultimo_elegido = 0;
                           }
                           else
                           {
                               ultimo_elegido = ultimo_elegido + 1;
                           }
                       }

                       "RN" | _=>{

                           //let num = rand::thread_rng().gen_range( 0, servidores_habilitados.len() - 1);
                           let mut num = self::rand::thread_rng().gen_range(0, servidores_habilitados.len() - 1);

                           channel_vector[i].tx.send(servidores_habilitados[num].to_string());
                       }
                    }
                }
                Err(TryRecvError::Empty) => {
                    //No hay nada para recibir
                }
                Err(TryRecvError::Disconnected) => unreachable!(),
            }
        }

        io::stdout().flush().expect("flushed");
        /*
        let pp : String  = channel_vector[0].rx.recv().unwrap();

        println! ("{:?}", pp);
        io::stdout().flush().expect("flushed");

        channel_vector[0].tx.send("http://emma.com".to_string());*/
    }
}
