#![feature(mpsc_select)]

extern crate rand;

use duplex;
use std::io::{self, Write};
use std::sync::mpsc::{TryRecvError};
use server_manager::rand::Rng;
use ipc_channel::router::RouterProxy;
use ipc_channel::ipc::{IpcSender, IpcReceiverSet};
use std::sync::Arc;

use std::collections::HashMap;

pub fn server_manager(mut channel_vector: Vec<duplex::DuplexStream>) {

    //Falta levantar la configuracion
    let estrategia = "RR"; //RoundRobin
//    let estrategia = "RN";//Random
    let mut ultimo_elegido = 0;
    let servidores_habilitados = vec!["http://192.0.0.1", "http://192.0.0.2", "http://192.0.0.3", "http://192.0.0.4"];


    let router = RouterProxy::new();
    let mut receiver_set = IpcReceiverSet::new().unwrap();

    let mut receiver_ids: Vec<u64> = Vec::new();
    let mut receiver_ids_map: HashMap<u64, IpcSender<String>> =  HashMap::new();

    for duplex_stream in channel_vector.into_iter() {
//        let duplex_stream = channel_vector.remove(i);
        let id = receiver_set.add(duplex_stream.rx).unwrap();
        receiver_ids_map.insert(id, duplex_stream.tx.clone());
    }


    loop {

        match receiver_set.select() {
            Ok(selection_result) => {
//                let (received_id, received_data) = selection_result.into_iter().next().unwrap().unwrap();
                for reference in selection_result.into_iter() {
                    let (received_id, received_data) = reference.unwrap();
                    let sender = receiver_ids_map.get(&received_id).unwrap();

                    let host_i = match estrategia {
                        "RR" => {
                            let selected_host = ultimo_elegido;//.copy();

                            if ultimo_elegido >= servidores_habilitados.len() - 1 {
                                ultimo_elegido = 0;
                            } else {
                                ultimo_elegido = ultimo_elegido + 1;
                            }
                            selected_host
                        },
                        "RN" | _ => {
                            self::rand::thread_rng().gen_range(0, servidores_habilitados.len() - 1)
                        }
                    };

                    let selected_host = servidores_habilitados[host_i];
                    info! ("Selected nro {}: {:?}", host_i, selected_host);
                    sender.send(selected_host.to_string());
                }
            },
            Err(e) => {
                error!("A problem occurs on server_manager channel.")
            }
        }

//        for i in 0..channel_vector.len()
//        {
//            let value = channel_vector[i].rx.try_recv();
//            match value {
//                Ok(id) => {
//                    info!("Valor: {}", id);
//
//                    //estrategia de asignacion de servers
//                    match estrategia
//                    {
//                       "RR"=>{
//
//                           info! ("Last selected: {:?}", ultimo_elegido);
//                           channel_vector[i].tx.send(servidores_habilitados[ultimo_elegido].to_string());
//
//                           if (ultimo_elegido >= servidores_habilitados.len() - 1)
//                           {
//                               ultimo_elegido = 0;
//                           }
//                           else
//                           {
//                               ultimo_elegido = ultimo_elegido + 1;
//                           }
//                       }
//
//                       "RN" | _=>{
//
//                           //let num = rand::thread_rng().gen_range( 0, servidores_habilitados.len() - 1);
//                           let mut num = self::rand::thread_rng().gen_range(0, servidores_habilitados.len() - 1);
//
//                           channel_vector[i].tx.send(servidores_habilitados[num].to_string());
//                       }
//                    }
//                }
//                Err(TryRecvError::Empty) => {
//                    //No hay nada para recibir
////                    info!("No hay nada...")
//                }
//                Err(TryRecvError::Disconnected) => unreachable!(),
//            }
//        }

        /*
        let pp : String  = channel_vector[0].rx.recv().unwrap();

        println! ("{:?}", pp);
        io::stdout().flush().expect("flushed");

        channel_vector[0].tx.send("http://emma.com".to_string());*/
    }
}
