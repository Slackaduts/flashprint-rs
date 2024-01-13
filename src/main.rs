// use num_derive::FromPrimitive;
// use num_traits::FromPrimitive;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

const START_BYTES: [u8; 2] = [0x7e, 0x4d];
const END_BYTES: [u8; 2] = [0x0d, 0x0a];
const RESPONSE_END_BYTES: [u8; 6] = [0x0d, 0x0a, 0x6f, 0x6b, 0x0d, 0x0a];


// #[derive(Clone, Copy, Debug, FromPrimitive)]
#[derive(Clone, Copy)]
#[repr(u16)]
enum PrinterCommand {
    MachineStatus = 119,
    MachineTemps = 105,
    PrintResume = 24,
    PrintPause = 25,
    PrintStatus = 27,
    SendFile = 28,
    SaveFile = 29,
    OpenFile = 23,
    GainControl = 601,
    ReleaseControl = 602,
    Unknown1 = 650,
    Unknown2 = 26,
}

impl fmt::Display for PrinterCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

impl PrinterCommand {
    fn build(&self, cmd_data: Option<String>) -> Vec<u8> {
        let mut cmd_bytes = START_BYTES.to_vec();

        cmd_bytes.extend_from_slice(&self.to_string().as_bytes()); //Convert M command number to string, then bytes, then add it to the command

        match cmd_data {
            Some(v) => {
                cmd_bytes.push(0x20); //Space
                cmd_bytes.extend_from_slice(v.as_bytes());
            }
            None => {}
        }

        cmd_bytes.extend_from_slice(&END_BYTES);

        return cmd_bytes;
    }

    fn send(&self, stream: &mut TcpStream, cmd: Vec<u8>) -> Option<String> {
        let write_res = stream.write(&cmd); //Send the command to the printer
        match write_res {
            //Check if the command was sent successfully
            Ok(_) => {}
            Err(e) => {
                println!("Failed to send command: {}", e);
                return None;
            }
        }

        let mut read_res = Vec::new();
        let mut buf = [0u8; 1];

        loop {
            //Read the response from the printer. If we get the "ok" response, stop reading. If we get a response that is too long, stop reading.
            match stream.read(&mut buf) {
                Ok(_) => {
                    read_res.push(buf[0]);
                    if read_res.ends_with(&RESPONSE_END_BYTES) {
                        break;
                    }

                    if read_res.len() >= 1024 { //The printer could theoretically send the max default payload size of 1460 bytes, but I highly doubt this will ever happen in a million years.
                        println!("Response too long");
                        break;
                    }
                }
                Err(e) => {
                    println!("Failed to read response: {}", e);
                    return None;
                }
            }
        }

        match String::from_utf8(read_res) { //Convert the response to a string
            Ok(res_str) => {
                return Some(res_str);
            }
            Err(e) => {
                println!("Failed to convert response to string: {}", e);
                return None;
            }
        }
    }

    fn send_cmd(&self, stream: &mut TcpStream, cmd_data: Option<String>) -> Option<String> {
        let cmd = self.build(cmd_data);
        return self.send(stream, cmd);
    }
}

fn main() {
    let mut stream = TcpStream::connect("192.168.1.10:8899").unwrap();

    println!("Connected to printer.");

    PrinterCommand::GainControl.send_cmd(&mut stream, Some(String::from("S1"))).unwrap();
    println!("1");

    PrinterCommand::MachineStatus.send_cmd(&mut stream, None).unwrap();
    println!("2");

    // println!("Machine Temps (105) Response: {}", PrinterCommand::MachineTemps.send_cmd(&mut stream, None).unwrap());

    //Write the contents of the chosen file to the stream.
    let file_path = r"C:\Users\Gabe\Documents\";
    let file_name = "Top Shell - Left Macro(1)_PLA_3h49m.gcode";
    let mut file = std::fs::File::open(format!("{}{}", file_path, file_name)).unwrap();

    // // Get the size of the chosen file
    let metadata = std::fs::metadata(format!("{}{}", file_path, file_name)).unwrap();
    let file_size = metadata.len();

    let file_data_str = format!(" {} 0:/user/{}", file_size.to_string(), file_name);

    // let cmd3 = PrinterCommand::SendFile;
    let response3 = PrinterCommand::SendFile.send_cmd(&mut stream, Some(file_data_str)).unwrap();
    println!("SEND FILE (28) Response: {}", response3);

    //Write the contents of the chosen file to the stream.
    std::io::copy(&mut file, &mut stream).unwrap();

    println!("Save File (29) Response: {}", PrinterCommand::SaveFile.send_cmd(&mut stream, None).unwrap());

    println!("Open file (23) Response: {}", PrinterCommand::OpenFile.send_cmd(&mut stream, Some(String::from(file_name))).unwrap());

    println!("Unknown 2 (26) Response: {}", PrinterCommand::Unknown2.send_cmd(&mut stream, None).unwrap());


    PrinterCommand::ReleaseControl.send_cmd(&mut stream, None).unwrap();
}