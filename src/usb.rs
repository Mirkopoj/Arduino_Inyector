use std::process::Child;
extern crate polling;
use polling::{Event, Poller};
use std::io::{BufRead, BufReader};

use std::sync::mpsc::Sender;

use crate::struct_paquete::Paquete;

pub fn pico_thread(mut child: Child, thread_tx: Sender<Paquete>) -> Result<(), std::io::Error> {
    let stdout = child.stdout.take().unwrap();
    let mut reader_out = BufReader::new(stdout);

    let key_out = 1;
    let mut out_closed = false;

    let poller = Poller::new().unwrap();
    poller
        .add(reader_out.get_ref(), Event::readable(key_out))
        .unwrap();

    let mut line = String::new();
    let mut events = Vec::new();

    for _ in 0..27 {
        // Wait for at least one I/O event.
        events.clear();
        poller.wait(&mut events, None).unwrap();

        for ev in &events {
            // stdout is ready for reading
            if ev.key == key_out {
                let len = match reader_out.read_line(&mut line) {
                    Ok(len) => len,
                    Err(e) => {
                        println!("stdout read returned error: {}", e);
                        0
                    }
                };
                if len == 0 {
                    println!("stdout closed (len is null)");
                    out_closed = true;
                    poller.delete(reader_out.get_ref()).unwrap();
                } else {
                    line.clear();
                    // reload the poller
                    poller
                        .modify(reader_out.get_ref(), Event::readable(key_out))
                        .unwrap();
                }
            }
        }

        if out_closed {
            println!("Stream closed, exiting process thread");
            break;
        }
    }

    loop {
        // Wait for at least one I/O event.
        events.clear();
        poller.wait(&mut events, None).unwrap();

        for ev in &events {
            // stdout is ready for reading
            if ev.key == key_out {
                let len = match reader_out.read_line(&mut line) {
                    Ok(len) => len,
                    Err(e) => {
                        println!("stdout read returned error: {}", e);
                        0
                    }
                };
                if len == 0 {
                    //println!("stdout closed (len is null)");
                    out_closed = true;
                    poller.delete(reader_out.get_ref()).unwrap();
                } else {
                    //println!("line: {}", line);
                    let line_clone = line.clone().replace("\n", "").replace("\r", "");
                    let com: u32 = line_clone.parse::<u32>().unwrap();
                    let paq = Paquete {
                        comando: ((com>>24) & 0x000000FF) as u8,
                        registro: ((com >> 16) & 0x000000FF) as u8,
                        valor: (com & 0x0000FFFF) as u16,
                    };
                    println!("\ncommando: {:x}", paq.comando);
                    println!("registro: {:x}", paq.registro);
                    println!("valor: {:x}", paq.valor);
                    thread_tx.send(paq).expect("Fallo en el canal (tx)");
                    line.clear();
                    // reload the poller
                    poller
                        .modify(reader_out.get_ref(), Event::readable(key_out))
                        .unwrap();
                }
            }
        }

        if out_closed {
            //println!("Stream closed, exiting process thread");
            break;
        }
    }

    Ok(())
}
