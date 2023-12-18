use std::net::TcpStream;
use std::io::{ Write, Read };
use std::vec;
use anyhow::Result;
use dialog::FileSelection;
use users::get_current_username;



const OPEN_FILE: (u16, u8) = (23, 90);
const PRINT_RESUME: (u16, u8) = (24, 23);
const PRINT_PAUSE: (u16, u8) = (25, 23);
const PRINT_STATUS: (u16, u8) = (27, 59);
const SEND_FILE: (u16, u8) = (28, 71);
const SAVE_FILE: (u16, u8) = (29, 42);

const MACHINE_TEMPS: (u16, u8) = (105, 59);
const POSITION_INFO: (u16, u8) = (114, 61);
const MACHINE_INFO: (u16, u8) = (115, 207);
const MACHINE_STATUS: (u16, u8) = (119, 152);


const GAIN_CONTROL: (u16, u8) = (601, 47);
const RELEASE_CONTROL: (u16, u8) = (602, 42);

#[derive(Clone, Copy)]
enum PrinterCommand {
    MachineStatus,
    MachineTemps,
    PrintResume,
    PrintPause,
    PrintStatus,
    SendFile,
    SaveFile,
    OpenFile,
    GainControl,
    ReleaseControl,
}

impl PrinterCommand {
    fn values(&self) -> (u16, u8) {
        match self {
            PrinterCommand::MachineStatus => MACHINE_STATUS,
            PrinterCommand::MachineTemps => MACHINE_TEMPS,
            PrinterCommand::PrintResume => PRINT_RESUME,
            PrinterCommand::PrintPause => PRINT_PAUSE,
            PrinterCommand::PrintStatus => PRINT_STATUS,
            PrinterCommand::SendFile => SEND_FILE,
            PrinterCommand::SaveFile => SAVE_FILE,
            PrinterCommand::OpenFile => OPEN_FILE,
            PrinterCommand::GainControl => GAIN_CONTROL,
            PrinterCommand::ReleaseControl => RELEASE_CONTROL,
        }
    }

    fn cmd_str(&self) -> String {
        let (a, _) = self.values();
        return a.to_string();
    }

    fn get_cmd_bytes(&self, extra_bytes: Option<Vec<u8>>) -> Vec<u8> {
        let start_bytes: [u8; 2] = [0x7e, 0x4d];
        let end_bytes: &[u8; 2] = &[0x0d, 0x0a];

        let cmd_str = self.cmd_str();
        let cmd_bytes: Vec<u8> = cmd_str.as_bytes().to_vec();

        let mut bytes = start_bytes.to_vec();
        bytes.extend_from_slice(&cmd_bytes);
        bytes.extend_from_slice(end_bytes);

        match extra_bytes {
            Some(v) => {
                bytes.extend_from_slice(&v);
            }
            None => {}
        }
    
        return bytes;
    }

    fn send_cmd(&self, stream: &mut TcpStream, extra_bytes: Option<Vec<u8>>) -> Result<String> {
        let bytes = self.get_cmd_bytes(extra_bytes);
        let result = stream.write(&bytes);

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to send command: {}", e);
                return Err(e.into());
            }
        }

        let mut data = vec![0; self.values().1 as usize];
        // Read the stream into data, and return the result.
        match stream.read_exact(&mut data) {
            Ok(_) => {
                match std::str::from_utf8(&data) {
                    Ok(v) => {
                        return Ok(v.to_string());
                    }
                    Err(e) => {
                        println!("Invalid UTF-8 sequence: {}", e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                println!("Failed to receive data: {}", e);
                Err(e.into())
            }
        }
    }
}


fn main() {
    let mut stream = TcpStream::connect("192.168.1.10:8899").unwrap();

    let mut cmd = PrinterCommand::GainControl;
    let mut response = cmd.send_cmd(&mut stream, None).unwrap();
    println!("Response: {}", response);

    let choice = dialog::FileSelection::new()
        .title("Select a file to print")
        .path(format!("C:/Users/{}/Documents", get_current_username()))
        .show()
        .expect("Failed to open file selection dialog");


    cmd = PrinterCommand::SendFile;
    response = cmd.send_cmd(&mut stream, None).unwrap();
    println!("Response: {}", response);

    //Write the contents of the chosen file to the stream.
    let mut file = std::fs::File::open(choice.path()).unwrap();
    // Get the size of the chosen file
    let metadata = std::fs::metadata(choice.path()).unwrap();
    let file_size = metadata.len();

    std::io::copy(&mut file, &mut stream).unwrap();

    let cmd3 = PrinterCommand::ReleaseControl;
    let response3 = cmd3.send_cmd(&mut stream).unwrap();
    println!("Response: {}", response3);



}