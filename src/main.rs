use std::net::TcpStream;
use std::io::{ Write, Read };
use std::vec;
use anyhow::Result;
// use dialog::FileSelection;
// use users::get_current_username;



const OPEN_FILE: (u16, u8) = (23, 78);
const PRINT_RESUME: (u16, u8) = (24, 23);
const PRINT_PAUSE: (u16, u8) = (25, 23);
const PRINT_STATUS: (u16, u8) = (27, 59);
const SEND_FILE: (u16, u8) = (28, 255);
const SAVE_FILE: (u16, u8) = (29, 42);

const MACHINE_TEMPS: (u16, u8) = (105, 59);
const POSITION_INFO: (u16, u8) = (114, 61);
const MACHINE_INFO: (u16, u8) = (115, 207);
const MACHINE_STATUS: (u16, u8) = (119, 152);


const GAIN_CONTROL: (u16, u8) = (601, 47);
const RELEASE_CONTROL: (u16, u8) = (602, 42);
const UNKNOWN1: (u16, u8) = (650, 39);

const UNKNOWN2: (u16, u8) = (26, 23);


const START_BYTES: [u8; 2] = [0x7e, 0x4d];
const END_BYTES: [u8; 2] = [0x0d, 0x0a];

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
    Unknown1,
    Unknown2,
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
            PrinterCommand::Unknown1 => UNKNOWN1,
            PrinterCommand::Unknown2 => UNKNOWN2,
        }
    }

    fn get_cmd_bytes(&self, extra_bytes: Option<Vec<u8>>) -> Vec<u8> {
        let start_bytes: [u8; 2] = [0x7e, 0x4d];
        let end_bytes: &[u8; 2] = &[0x0d, 0x0a];

        let cmd_str = self.cmd_str();
        let cmd_bytes: Vec<u8> = cmd_str.as_bytes().to_vec();

        let mut bytes = start_bytes.to_vec();
        bytes.extend_from_slice(&cmd_bytes);
        

        match extra_bytes {
            Some(v) => {
                bytes.extend_from_slice(&v);
            }
            None => {}
        }

        bytes.extend_from_slice(end_bytes);

        //Print bytes as utf8
        match std::str::from_utf8(&bytes) {
            Ok(v) => println!("Bytes as utf8: {}", v),
            Err(e) => println!("Invalid UTF-8 sequence: {}", e),
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
        match stream.read(&mut data) {
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

    fn build_cmd(&self, cmd_data: Option<String>) -> String {
        let mut cmd_str = START_BYTES.to_ascii_lowercase();

        match cmd_data {
            Some(v) => {
                cmd_str.push_str(&v);
            }
            None => {}
        }

        


        return cmd_str;
    }
}


fn main() {
    let mut stream = TcpStream::connect("192.168.1.10:8899").unwrap();

    let cmd = PrinterCommand::GainControl;
    let response = cmd.send_cmd(&mut stream, Some(" S1".as_bytes().to_vec())).unwrap();
    println!("GAIN_CONTROL (601) Response: {}", response);

    //Write the contents of the chosen file to the stream.
    let file_path = r"C:\Users\Gabe\Documents\";
    let file_name = "Rocktopus.gcode";
    let mut file = std::fs::File::open(format!("{}{}", file_path, file_name)).unwrap();

    // Get the size of the chosen file
    let metadata = std::fs::metadata(format!("{}{}", file_path, file_name)).unwrap();
    let file_size = metadata.len();

    let file_data_str = format!(" {} 0:/user/{}", file_size.to_string(), file_name);


    let cmd3 = PrinterCommand::SendFile;
    let response3 = cmd3.send_cmd(&mut stream, Some(file_data_str.as_bytes().to_vec())).unwrap();
    println!("SEND FILE (28) Response: {}", response3);


    //Write the contents of the chosen file to the stream.
    std::io::copy(&mut file, &mut stream).unwrap();

    // std::thread::sleep(std::time::Duration::from_secs(1));

    let response4 = PrinterCommand::SaveFile.send_cmd(&mut stream, None).unwrap();
    println!("SAVE FILE (29) Response: {}", response4);


    let mut file_name_bytes = vec![0x20];
    file_name_bytes.extend_from_slice(&file_name.as_bytes());
    let response5 = PrinterCommand::OpenFile.send_cmd(&mut stream, Some(file_name_bytes)).unwrap();
    println!("OPEN FILE (23) Response: {}", response5);


    let cmd6 = PrinterCommand::Unknown2;
    let response6 = cmd6.send_cmd(&mut stream, None).unwrap();
    println!("UNKNOWN 2 (26) Response: {}", response6);


    let cmd7 = PrinterCommand::ReleaseControl;
    let response7 = cmd7.send_cmd(&mut stream, None).unwrap();
    println!("RELEASE CONTROL (602) Response: {}", response7);



}