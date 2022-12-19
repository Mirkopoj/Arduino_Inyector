use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, 
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
//use std::io::prelude::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::{thread, time};
use tui::{
    backend::CrosstermBackend,
    layout::Constraint, //{Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState}, //Widget
    Terminal,
};

mod struct_paquete;
use crate::struct_paquete::Paquete;

mod usb;
use crate::usb::pico_thread;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

enum CEvent<I> {
    Input(I),
    Tick,
}

const PRINTS:bool = false;

fn main() -> io::Result<()> {
    //Launch picocom
    let mut child = match Command::new("picocom")
        .arg("/dev/ttyACM1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("Falló picocom: {}", why),
        Ok(child) => child,
    };
    let child_id = child.id() as i32;
    let mut picoin = child.stdin.take().expect("No se abrió el stdin");

    //thread usb para leer de picocom
    let (reader_tx, reader_rx): (Sender<Paquete>, Receiver<Paquete>) = mpsc::channel();
    let pico_reader = thread::spawn({
        move || {
            pico_thread(child, reader_tx).unwrap();
        }
    });

    //thread write para enviar a picocom
    let (writer_tx, writer_rx): (Sender<Paquete>, Receiver<Paquete>) = mpsc::channel();
    let pico_writer = thread::spawn({
        move || {
            thread::sleep(time::Duration::from_millis(2000));
            loop {
                let paq = writer_rx.recv().expect("Falló el thread pico_writer");
                let mut msg: u32 = match paq.comando {
                    0x25 => 0x25000000 | (paq.registro as u32) << 16 | paq.valor as u32,
                    0x3C => 0x3C000000 | (paq.registro as u32) << 16,
                    0x29 => 0x29000000 | (paq.registro as u32) << 16 | paq.valor as u32,
                    0x37 => 0x37000000 | (paq.registro as u32) << 16 | paq.valor as u32,
                    0xFF => {
                        break;
                    }
                    _ => 0xFFFFFFFF,
                };
                let msgh = msg & 0xFFFF0000;
                let msgl = msg & 0x0000FFFF;
                msg |= (msgh.count_ones()%2)<<31;
                msg |= (msgl.count_ones()%2)<<15;
                let out = format!("{:010}", msg);
                if PRINTS {
                    println!("\nout: {}", out);
                    println!("\nmsg: {:08X}", msg);
                }
                picoin.write_all(out.as_bytes()).expect(format!("No salió {:08X}", msg&0x7FFFFFFF).as_str());
                thread::sleep(time::Duration::from_millis(50));
            }
        }
    });

    //thread tui para input de eventos
    let (tui_tx, tui_rx) = mpsc::channel();
    let tick_rate = time::Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = time::Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| time::Duration::from_secs(0));
            if event::poll(timeout).expect("poll works") {
                if let Event::Key(key) = event::read().expect("can read events") {
                    tui_tx.send(CEvent::Input(key)).expect("can send events");
                }
            }
            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tui_tx.send(CEvent::Tick) {
                    last_tick = time::Instant::now();
                }
            }
        }
    });

    let mut registros: [u16; 20] = [0; 20];
    let mut señal: [bool; 12] = [false; 12];
    let analogicas: [u16; 8] = [0; 8];
    let mut external: [u16; 2] = [0; 2];

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut reg_table_state = TableState::default();
    reg_table_state.select(Some(0));
    let mut sig_table_state = TableState::default();
    sig_table_state.select(None);
    let mut ext_table_state = TableState::default();
    ext_table_state.select(None);

    let mut registro_buffer = 0;
    let mut ext_buffer = 0;
    let mut escribiendo_registro = false;
    let mut selected_bit = 16;

    pico_update(writer_tx.clone(), 0, 20);

    loop {
        terminal.draw(|f| {
            let mut sizesig = f.size();
            let mut sizereg = f.size();
            let mut sizeext = f.size();

            sizereg.width /= 2;
            sizesig.width /= 2;
            sizesig.x = sizereg.width;
            sizereg.height = (sizereg.height*7)/10;
            sizesig.height = (sizesig.height*7)/10;
            sizeext.height = (sizeext.height*3)/10;
            sizeext.y = sizereg.height;

            //TABLA 1 - REGISTROS

            let tablereg = Table::new(vec![
                Row::new(vec![
                    Cell::from("00").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("STATUS"),
                    Cell::from(
                        "P  M  HR HT HD G4 G3 G2 G1 RP DP UD OD DC OT OC \n".to_string()
                            + &(format!("{:016b}", registros[0])
                                .chars()
                                .enumerate()
                                .flat_map(|(_i, c)| {
                                    Some(c)
                                        .into_iter()
                                        .chain(std::iter::once(' '))
                                        .chain(std::iter::once(' '))
                                })
                                .collect::<String>())[..],
                    ),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ])
                .height(2),
                Row::new(vec![
                    Cell::from("01").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("CONTROL"),
                    Cell::from(vec![
                        Spans::from(vec![
                            Span::styled(
                                "P  ",
                                Style::default().fg(if selected_bit == 15 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "0  ",
                                Style::default().fg(if selected_bit == 14 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "0  ",
                                Style::default().fg(if selected_bit == 13 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "0  ",
                                Style::default().fg(if selected_bit == 12 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "0  ",
                                Style::default().fg(if selected_bit == 11 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "0  ",
                                Style::default().fg(if selected_bit == 10 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "ST ",
                                Style::default().fg(if selected_bit == 9 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "LD ",
                                Style::default().fg(if selected_bit == 8 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "SD ",
                                Style::default().fg(if selected_bit == 7 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "SR ",
                                Style::default().fg(if selected_bit == 6 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "AR ",
                                Style::default().fg(if selected_bit == 5 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "RP ",
                                Style::default().fg(if selected_bit == 4 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "OD ",
                                Style::default().fg(if selected_bit == 3 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "DC ",
                                Style::default().fg(if selected_bit == 2 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "OT ",
                                Style::default().fg(if selected_bit == 1 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                            Span::styled(
                                "OC\n",
                                Style::default().fg(if selected_bit == 0 {
                                    Color::LightRed
                                } else {
                                    Color::White
                                }),
                            ),
                        ]),
                        Spans::from(vec![Span::styled(
                            format!("{:016b}", registros[1])
                                .chars()
                                .enumerate()
                                .flat_map(|(_i, c)| {
                                    Some(c)
                                        .into_iter()
                                        .chain(std::iter::once(' '))
                                        .chain(std::iter::once(' '))
                                })
                                .collect::<String>(),
                            Style::default().fg(if selected_bit != 16 {
                                Color::LightRed
                            } else {
                                Color::White
                            }),
                        )]),
                    ]),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ])
                .height(2),
                Row::new(vec![
                    Cell::from("02").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF FWD"),
                    Cell::from(registros[2].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("03").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF RFL"),
                    Cell::from(registros[3].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("04").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF INPUT"),
                    Cell::from(registros[4].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("05").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("TEMPERATURE"),
                    Cell::from(registros[5].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("06").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN1 CURRENT"),
                    Cell::from(registros[6].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("07").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN2 CURRENT"),
                    Cell::from(registros[7].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("08").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN3 CURRENT"),
                    Cell::from(registros[8].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("09").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN4 CURRENT"),
                    Cell::from(registros[9].to_string()),
                    Cell::from("R").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from("10").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("OVER TEMPERATURE THRESHOLD"),
                    Cell::from(registros[10].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(10)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("11").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("TEMPERATURE HYSTERESIS THRESHOLD"),
                    Cell::from(registros[11].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(11)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("12").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("OVER CURRENT THRESHOLD"),
                    Cell::from(registros[12].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(12)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("13").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("DUTY CYCLE PROTECTION THRESHOLD"),
                    Cell::from(registros[13].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(13)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("14").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("PULSE LENGTH PROTECTION THRESHOLD"),
                    Cell::from(registros[14].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(14)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("15").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("OVER DRIVE PROTECTION THRESHOLD"),
                    Cell::from(registros[15].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(15)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("16").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("UNDER DRIVE PROTECTION THRESHOLD"),
                    Cell::from(registros[16].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(16)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("17").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF FWD ALARM THRESHOLD"),
                    Cell::from(registros[17].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(17)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("18").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF RFL ALARM THRESHOLD"),
                    Cell::from(registros[18].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(18)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from("19").style(Style::default().fg(Color::LightCyan)),
                    Cell::from("SSPA SERIAL NUMBER"),
                    Cell::from(registros[19].to_string()).style(Style::default().fg(
                        if escribiendo_registro & (reg_table_state.selected() == Some(19)) {
                            Color::LightRed
                        } else {
                            Color::White
                        },
                    )),
                    Cell::from("R/W").style(Style::default().fg(Color::LightGreen)),
                ]),
            ])
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(vec!["Address", "Nombre", "Registro", "Access"])
                    .style(Style::default().fg(Color::LightYellow))
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title("-Registros")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .widths(&[
                Constraint::Length(7),
                Constraint::Length(33),
                Constraint::Length(48),
                Constraint::Length(10),
            ])
            .column_spacing(1)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("");
            f.render_stateful_widget(tablereg, sizereg, &mut reg_table_state);

            //TABLA 2 - SEÑALES

            let tablesen = Table::new(vec![
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[0])).style(Style::default().fg(
                        if señal[0] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("POWER ENABLE").style(Style::default().fg(Color::White)),
                    Cell::from("ENTRADA").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[1])).style(Style::default().fg(
                        if señal[1] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("SSPA ACTIVE").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[0]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF FWD").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[1]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF RFL").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[2]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("RF INPUT").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[3]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("TEMPERATURE").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[4]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN1 CURRENT").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[5]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN2 CURRENT").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[6]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN3 CURRENT").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("{:010x}", analogicas[7]))
                        .style(Style::default().fg(Color::LightCyan)),
                    Cell::from("GAN4 CURRENT").style(Style::default().fg(Color::White)),
                    Cell::from("ANALOGICA").style(Style::default().fg(Color::LightCyan)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[2])).style(Style::default().fg(
                        if señal[2] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("GAN1 BK").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[3])).style(Style::default().fg(
                        if señal[3] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("GAN2 BK").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[4])).style(Style::default().fg(
                        if señal[4] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("GAN3 BK").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[5])).style(Style::default().fg(
                        if señal[5] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("GAN4 BK").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[6])).style(Style::default().fg(
                        if señal[6] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("OVD").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[7])).style(Style::default().fg(
                        if señal[7] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("TnR").style(Style::default().fg(Color::White)),
                    Cell::from("ENTRADA").style(Style::default().fg(Color::LightRed)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[8])).style(Style::default().fg(
                        if señal[8] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("°C MAX").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[9])).style(Style::default().fg(
                        if señal[9] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("RFLHI").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[11])).style(Style::default().fg(
                        if señal[10] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("EXT TNR").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
                Row::new(vec![
                    Cell::from(format!("[{ }]", señal[10])).style(Style::default().fg(
                        if señal[11] {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        },
                    )),
                    Cell::from("EXT POWER ENABLE").style(Style::default().fg(Color::White)),
                    Cell::from("SALIDA").style(Style::default().fg(Color::LightGreen)),
                ]),
            ])
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(vec!["Output", "Señal", "Tipo"])
                    .style(Style::default().fg(Color::LightYellow))
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title("-Señales")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(10),
            ])
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // If you wish to highlight a row in any speci:wfic way when it is selected...
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            // ...and potentially show a symbol in front of the selection.
            .highlight_symbol("");
            f.render_stateful_widget(tablesen, sizesig, &mut sig_table_state);


            //TABLA 3 - SEÑALES

            let tableext = Table::new(vec![
                Row::new(vec![
                    Cell::from("TnR"),
                    Cell::from(external[0].to_string()),
                ]),
                Row::new(vec![
                    Cell::from("POWER ENABLE"),
                    Cell::from(external[1].to_string()),
                ]),
            ])
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(vec!["Nombre", "Tiempo (N * 62,5 nS) (N=0 => APAGADO, N=65535 => ENCENDIDO CONSTANTE)"])
                    .style(Style::default().fg(Color::LightYellow))
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title("-External")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .widths(&[
                Constraint::Length(14),
                Constraint::Length(70),
            ])
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // If you wish to highlight a row in any speci:wfic way when it is selected...
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            // ...and potentially show a symbol in front of the selection.
            .highlight_symbol("");
            f.render_stateful_widget(tableext, sizeext, &mut ext_table_state);
        })?;


        match tui_rx.recv().unwrap() {
            CEvent::Input(event) => match event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    break;
                }
                KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Down => {
                    if !escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                let ind = if n == 19 { 0 } else { n + 1 };
                                reg_table_state.select(Some(ind));
                            }
                            None => {}
                        };
                        match sig_table_state.selected() {
                            Some(n) => {
                                let ind = if n == 19 { 0 } else { n + 1 };
                                sig_table_state.select(Some(ind));
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                ext_table_state.select(Some((n+1)%2));
                            },
                            None => {},
                        };
                    };
                }
                KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Up => {
                    if !escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                let ind = if n == 0 { 19 } else { n - 1 };
                                reg_table_state.select(Some(ind));
                            }
                            None => {}
                        };
                        match sig_table_state.selected() {
                            Some(n) => {
                                let ind = if n == 0 { 19 } else { n - 1 };
                                sig_table_state.select(Some(ind));
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                ext_table_state.select(Some((n+1)%2));
                            },
                            None => {},
                        };
                    };
                }
                KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Right => {
                    if !escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(_) => {
                                reg_table_state.select(None);
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(_) => {
                                ext_table_state.select(None);
                            },
                            None => {},
                        };
                        match sig_table_state.selected() {
                            Some(_) => {
                                ext_table_state.select(Some(0));
                                sig_table_state.select(None);
                            }
                            None => {
                                sig_table_state.select(Some(0));
                            }
                        };
                    } else {
                        if reg_table_state.selected() == Some(1) {
                            if selected_bit != 0 {
                                selected_bit -= 1;
                            }
                        }
                    };
                }
                KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Left => {
                    if !escribiendo_registro {
                        match ext_table_state.selected() {
                            Some(_) => {
                                ext_table_state.select(None);
                            },
                            None => {},
                        };
                        match sig_table_state.selected() {
                            Some(_) => {
                                sig_table_state.select(None);
                            }
                            None => {}
                        };
                        match reg_table_state.selected() {
                            Some(_) => {
                                ext_table_state.select(Some(0));
                                reg_table_state.select(None);
                            }
                            None => {
                                reg_table_state.select(Some(0));
                            }
                        };
                    } else {
                        if reg_table_state.selected() == Some(1) {
                            if selected_bit != 15 {
                                selected_bit += 1;
                            }
                        }
                    };
                }
                KeyCode::Char(' ') => {
                    if !escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                match n {
                                    1 => {
                                        selected_bit = 0;
                                        registro_buffer = registros[n];
                                        registros[n] = 0;
                                        escribiendo_registro = true;
                                    }
                                    0 | 2..=9 => {}
                                    _ => {
                                        registro_buffer = registros[n];
                                        registros[n] = 0;
                                        escribiendo_registro = true;
                                    }
                                };
                            }
                            None => {}
                        };
                        match sig_table_state.selected() {
                            Some(n) => {
                                match n {
                                    1 => arduino_pin_toggle(
                                        señal[1] ^ true,
                                        2,
                                        writer_tx.clone(),
                                    ),
                                    0 | 2..=9 | 15 => {}
                                    16 | 17 => arduino_pin_toggle(
                                        señal[n - 8] ^ true,
                                        (n - 8) as u8,
                                        writer_tx.clone(),
                                    ),
                                    18 | 19 => arduino_pin_toggle(
                                        señal[n - 8] ^ true,
                                        //(n + 30) as u8,
                                        (n - 7) as u8,
                                        writer_tx.clone(),
                                    ),
                                    _ => arduino_pin_toggle(
                                        señal[n - 8] ^ true,
                                        (n - 7) as u8,
                                        writer_tx.clone(),
                                    ),
                                };
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                ext_buffer = external[n];
                                external[n] = 0;
                                escribiendo_registro = true;
                            }
                            None => {}
                        };
                    } else {
                        if reg_table_state.selected() == Some(1) {
                            registros[1] ^= 1 << selected_bit;
                        }
                    };
                }
                KeyCode::Enter => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                escribiendo_registro = false;
                                pico_write(n as u8, registros[n], writer_tx.clone());
                                registros[n] = registro_buffer;
                                if n == 1 {
                                    selected_bit = 16;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                escribiendo_registro = false;
                                pico_ext_write(n as u8, external[n], writer_tx.clone());
                                external[n] = ext_buffer;
                            }
                            None => {}
                        };
                    } else {
                        match reg_table_state.selected() {
                            Some(n) => {
                                pico_update(writer_tx.clone(), n as u8, n as u8);
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Esc => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                escribiendo_registro = false;
                                registros[n] = registro_buffer;
                                if n == 1 {
                                    selected_bit = 16;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                escribiendo_registro = false;
                                external[n] = ext_buffer;
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Backspace => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                registros[n] /= 10;
                            }
                            None => {}
                        };
                    };
                    match ext_table_state.selected() {
                        Some(n) => {
                            external[n] /= 10;
                        }
                        None => {}
                    };
                }
                KeyCode::Char('0') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('1') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                    registros[n] += 1;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                    external[n] += 1;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('2') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                    registros[n] += 2;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                    external[n] += 2;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('3') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                    registros[n] += 3;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                    external[n] += 3;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('4') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                    registros[n] += 4;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                    external[n] += 4;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('5') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6554 {
                                    registros[n] *= 10;
                                    registros[n] += 5;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6554 {
                                    external[n] *= 10;
                                    external[n] += 5;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('6') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6553 {
                                    registros[n] *= 10;
                                    registros[n] += 6;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6553 {
                                    external[n] *= 10;
                                    external[n] += 6;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('7') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6553 {
                                    registros[n] *= 10;
                                    registros[n] += 7;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6553 {
                                    external[n] *= 10;
                                    external[n] += 7;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('8') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6553 {
                                    registros[n] *= 10;
                                    registros[n] += 8;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6553 {
                                    external[n] *= 10;
                                    external[n] += 8;
                                }
                            }
                            None => {}
                        };
                    };
                }
                KeyCode::Char('9') => {
                    if escribiendo_registro {
                        match reg_table_state.selected() {
                            Some(n) => {
                                if registros[n] < 6553 {
                                    registros[n] *= 10;
                                    registros[n] += 9;
                                }
                            }
                            None => {}
                        };
                        match ext_table_state.selected() {
                            Some(n) => {
                                if external[n] < 6553 {
                                    external[n] *= 10;
                                    external[n] += 9;
                                }
                            }
                            None => {}
                        };
                    };
                }
                _ => {}
            },
            CEvent::Tick => {}
        }

        loop {
            let paq = match reader_rx.try_recv() {
                Ok(pak) => pak,
                Err(why) => {
                    if why == TryRecvError::Empty {
                        break;
                    } else {
                        panic!("reader_tx terminated")
                    }
                }
            };
            match paq.comando & 0x7F {
                0x25 | 0x3C => { registros[paq.registro as usize] = paq.valor & 0x7FFF; }
                0x29 => {señal[
                    match paq.registro {
                        2 => 1,
                        3..=7 => (paq.registro - 1) as usize,
                        8 | 9 => (paq.registro) as usize,
                        //48 | 49 => (paq.registro - 38) as usize,
                        12 | 11 => (paq.registro - 1) as usize,
                        _ => 0,
                    }
                ] = paq.valor == 0xFFFF; }
                _ => { }
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    //termina los threads
    signal::kill(Pid::from_raw(child_id), Signal::SIGKILL).unwrap();
    writer_tx
        .send(Paquete {
            comando: 0xFF,
            registro: 0xFF,
            valor: 0,
        })
        .unwrap();
    pico_reader.join().unwrap();
    pico_writer.join().unwrap();

    Ok(())
}

fn pico_write(reg: u8, dato: u16, writer_tx: Sender<Paquete>) {
    let paq = Paquete {
        comando: 0x25,
        registro: reg,
        valor: dato,
    };
    writer_tx.send(paq).expect("Falló fn pico_write");
    pico_update(writer_tx.clone(), 0, 10);
    if reg==1 && dato&0x0100 == 0x0100 {
        pico_update(writer_tx.clone(), 10, 20);
    };
}

fn pico_ext_write(reg: u8, dato: u16, writer_tx: Sender<Paquete>) {
    let paq = Paquete {
        comando: 0x37,
        registro: reg+11,
        valor: dato,
    };
    writer_tx.send(paq).expect("Falló fn pico_write");
}

fn arduino_pin_toggle(valor: bool, pin: u8, writer_tx: Sender<Paquete>) {
    let paq = Paquete {
        comando: 0x29,
        registro: pin,
        valor: if valor { 0xFFFF } else { 0x0000 },
    };
    writer_tx.send(paq).expect("Falló fn arduino_pin_toggle");
    pico_update(writer_tx.clone(), 0, 10);
}

fn pico_update(writer_tx: Sender<Paquete>, inicio:u8, fin:u8){
    for i in inicio..fin {
        let paq = Paquete {
            comando: 0x3C,
            registro: i,
            valor: 0,
        };
        writer_tx.send(paq).expect("Falló fn pico_update");
    }
}
