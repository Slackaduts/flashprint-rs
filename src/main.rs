use std::net::TcpStream;
use std::io::{ Write, Read };
use std::{ vec, fmt };
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const START_BYTES: [u8; 2] = [0x7e, 0x4d];
const END_BYTES: [u8; 2] = [0x0d, 0x0a];

// #[derive(Clone, Copy, Display, Debug)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u16)]
enum PrinterCommand {
    MachineStatus = 119,
    MachineTemps = 105,
    PrintResume = 24,
    PrintPause = 25,
    PrintStatus= 27,
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
        match PrinterCommand::from_u16(*self as u16) {
            Some(command) => write!(f, "{}", command as u16),
            None => write!(f, "Unknown command"),
        }
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
        let write_res = stream.write(&cmd);

        match write_res {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to send command: {}", e);
                return None;
            }
        }

        let mut read_res = vec![0; 1460];
        match stream.read_to_end(&mut read_res) {
            Ok(_) => {
                println!("Response: {:?}", read_res);
            }
            Err(e) => {
                println!("Failed to read response: {}", e);
                return None;
            }
        }

        match String::from_utf8(read_res) {
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

    let response = PrinterCommand::MachineStatus.send_cmd(&mut stream, None);

    println!("Got a response");

    match response {
        Some(v) => {
            println!("Machine Status (119) Response: {}", v);
        }
        None => {
            println!("Failed to get response");
        }
    }

    // let cmd = PrinterCommand::GainControl;

    // let response = cmd.send_cmd(&mut stream, Some("S1")).unwrap();
    // println!("GAIN_CONTROL (601) Response: {}", response);

    // //Write the contents of the chosen file to the stream.
    // let file_path = r"C:\Users\Gabe\Documents\";
    // let file_name = "Rocktopus.gcode";
    // let mut file = std::fs::File::open(format!("{}{}", file_path, file_name)).unwrap();

    // // Get the size of the chosen file
    // let metadata = std::fs::metadata(format!("{}{}", file_path, file_name)).unwrap();
    // let file_size = metadata.len();

    // let file_data_str = format!(" {} 0:/user/{}", file_size.to_string(), file_name);


    // let cmd3 = PrinterCommand::SendFile;
    // let response3 = cmd3.send_cmd(&mut stream, Some(file_data_str.as_bytes().to_vec())).unwrap();
    // println!("SEND FILE (28) Response: {}", response3);


    // //Write the contents of the chosen file to the stream.
    // std::io::copy(&mut file, &mut stream).unwrap();

    // // std::thread::sleep(std::time::Duration::from_secs(1));

    // let response4 = PrinterCommand::SaveFile.send_cmd(&mut stream, None).unwrap();
    // println!("SAVE FILE (29) Response: {}", response4);


    // let mut file_name_bytes = vec![0x20];
    // file_name_bytes.extend_from_slice(&file_name.as_bytes());
    // let response5 = PrinterCommand::OpenFile.send_cmd(&mut stream, Some(file_name_bytes)).unwrap();
    // println!("OPEN FILE (23) Response: {}", response5);


    // let cmd6 = PrinterCommand::Unknown2;
    // let response6 = cmd6.send_cmd(&mut stream, None).unwrap();
    // println!("UNKNOWN 2 (26) Response: {}", response6);


    // let cmd7 = PrinterCommand::ReleaseControl;
    // let response7 = cmd7.send_cmd(&mut stream, None).unwrap();
    // println!("RELEASE CONTROL (602) Response: {}", response7);



}