#![allow(dead_code, unused_imports)]

use std::io::{self, Read, Cursor, Seek};
use std::fs;

#[derive(Debug, Default)]
struct Header {
    header: String,

    dem_protocol: i32,
    net_protocol: i32,

    server: String,
    client: String,
    map: String,
    dir: String,

    time: f32,
    ticks: i32,
    frames: i32,
    signon: i32
}

#[derive(Debug)]
struct Demo {
    file: Cursor<Vec<u8>>,

    header: Header,
    cmds: Vec<(i32, String)>,
    ticks: Option<i32>,
}

impl Demo {
    pub fn new(path: &str) -> io::Result<Self> {
        let mut demo = vec![];
        let mut demo_file = fs::File::open(path)?;
        demo_file.read_to_end(&mut demo)?;

        Ok(Self {
            file: Cursor::new(demo),

            header: Header::default(),
            cmds: vec![],
            ticks: None,
        })
    }

    pub fn parse(&mut self) -> io::Result<i32> {
        if let Some(ticks) = self.ticks {
            return Ok(ticks);
        }

        self.read_header()?;
        self.read_body()?;

        Ok(self.ticks.unwrap())
    }

    fn seek(&mut self, len: i64) -> io::Result<u64> {
        self.file.seek(io::SeekFrom::Current(len))
    }

    fn read_body(&mut self) -> io::Result<()> {
        let mut current_tick = 0;
        loop {
            let cmd = self.read_cmd()?;
            match self.ticks {
                Some(t) if current_tick < t => break,
                _ => self.ticks = Some(current_tick),
            }

            match cmd {
                1 => {
                    self.seek(self.header.signon as i64 - 1)?;
                }
                2 => {
                    current_tick = self.read_i32()?;
                    self.seek(4)?;
                    self.seek(12 + 68)?;
                    let data_size = self.read_i32()?;
                    self.seek(data_size as i64)?;
                }
                3 => {
                    current_tick = self.read_i32()?;
                }
                4 => {
                    current_tick = self.read_i32()?;
                    let data_size = self.read_i32()?;
                    let cmd = self.read_string(data_size as usize)?;
                    self.cmds.push((current_tick, cmd));
                }
                5 => {
                    current_tick = self.read_i32()?;
                    self.seek(4)?;
                    let data_size = self.read_i32()?;
                    self.seek(data_size as i64)?;
                }
                6 => {
                    eprintln!("Command 6: dem_datatables unimplemented!");
                    break;
                }
                7 => {
                    break;
                }
                8 => {
                    current_tick = self.read_i32()?;
                    let data_size = self.read_i32()?;
                    self.seek(data_size as i64)?;
                }
                n => unimplemented!("No command implemented for {} (pos: {})", n, self.file.position())
            }
        }

        Ok(())
    }

    fn read_header(&mut self) -> io::Result<()> {
        self.header.header = self.read_string(8)?;

        self.header.dem_protocol = self.read_i32()?;
        self.header.net_protocol = self.read_i32()?;

        self.header.server = self.read_string(260)?;
        self.header.client = self.read_string(260)?;
        self.header.map = self.read_string(260)?;
        self.header.dir = self.read_string(260)?;

        self.header.time = self.read_f32()?;
        self.header.ticks = self.read_i32()?;
        self.header.frames = self.read_i32()?;
        self.header.signon = self.read_i32()?;

        Ok(())
    }

    fn read_string(&mut self, len: usize) -> io::Result<String> {
        let mut buf = vec![0; len];
        self.file.read_exact(&mut buf)?;
        let s = String::from_utf8_lossy(&buf);

        Ok(s.to_string())
    }

    fn read_i32(&mut self) -> io::Result<i32> {
        let mut buf = [0; 4];
        self.file.read_exact(&mut buf)?;
        let i = i32::from_le_bytes(buf);

        Ok(i)
    }

    fn read_f32(&mut self) -> io::Result<f32> {
        let mut buf = [0; 4];
        self.file.read_exact(&mut buf)?;
        // Why the fuck isn't from_le_bytes available for f32?
        // https://doc.rust-lang.org/stable/std/primitive.f32.html#method.from_bits
        let i = u32::from_le_bytes(buf);
        let i = f32::from_bits(i);

        Ok(i)
    }

    fn read_cmd(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.file.read_exact(&mut buf)?;
        let i = u8::from_le_bytes(buf);

        Ok(i)
    }
}

fn main() {
    let mut demo = Demo::new(
        &std::env::args().nth(1).expect("No demo supplied")
    ).unwrap();

    println!("{}", demo.parse().unwrap());
}
