use log::{info, trace};
use std::env;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::thread;

const DEFAULT_BUF_SIZE: usize = 1024;

/*
 *
 * <Typical TCP Connection>
 *   Local TCP Client ---> Remote TCP Server
 * <Proposed Method>
 *   Local TCP Client ---> Redirector TCP Server -------> Redirect TCP Client ------> Remote TCP Server
 * */

fn handle_local_client(mut local: TcpStream, remote_ip: Ipv4Addr, remote_port: u16) {
    info!("New client connected! {:?}", local);

    let mut remote = TcpStream::connect(format!("{}:{}", remote_ip, remote_port)).unwrap();
    trace!("Connect to remote is ok!");

    let local_tcp_handler = {
        let mut client = local.try_clone().unwrap();
        let mut remote = remote.try_clone().unwrap();
        thread::spawn(move || {
            let mut buf: [u8; DEFAULT_BUF_SIZE] = [0; DEFAULT_BUF_SIZE];
            loop {
                let n = client.read(&mut buf).expect("read from local client");
                trace!("[CLIENT] read {} bytes from client", n);
                if n == 0 {
                    trace!("[CLIENT] DISCONNECT!");
                    break;
                }

                let _w = remote.write(&buf[..n]).unwrap();
                trace!("[CLIENT] write {} bytes to remote", _w);
            }
        })
    };

    let mut buf: [u8; DEFAULT_BUF_SIZE] = [0; DEFAULT_BUF_SIZE];
    loop {
        let n = remote.read(&mut buf).expect("read from remote server");
        trace!("[REMOTE] read {} bytes from remote", n);

        if n == 0 {
            trace!("[REMOTE] DISCONNECT!");
            break;
        }

        let _w = local.write(&buf[..n]).unwrap();
        trace!("[REMOTE] write {} bytes to client", _w);
    }

    local_tcp_handler.join().unwrap();

    info!("transfer thread is finished!");
}

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: <LOCAL PORT> <REMOTE HOST> <REMOTE PORT>");
        return;
    }

    let local_port: u16 = args[1].parse().unwrap();
    let remote_ip: Ipv4Addr = args[2].parse().unwrap();
    let remote_port: u16 = args[3].parse().unwrap();

    let listner = TcpListener::bind(format!("0.0.0.0:{}", local_port)).unwrap();
    info!("service started");

    for client in listner.incoming() {
        let client = client.unwrap().try_clone().unwrap();
        thread::spawn(move || handle_local_client(client, remote_ip, remote_port));
    }
}
