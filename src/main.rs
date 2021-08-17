use std::{
    io::{self, stdout, Read, Write},
    thread::sleep,
    time::Duration,
};
use tello::{
    drone_state::{FlightData, LightInfo},
    Drone, Message, Package, PackageData, ResponseMsg,
};
use termion::{async_stdin, clear, cursor, raw::IntoRawMode};

fn main() -> Result<(), io::Error> {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin();

    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
    stdout.flush()?;

    let mut drone = Drone::new("192.168.10.1:8889");
    drone.connect(11111);

    let mut fl_data: Option<FlightData> = None;
    let mut li_info: Option<LightInfo> = None;
    let mut connected: bool = false;

    let mut upwards: f32 = 0.0f32;
    let mut forwards: f32 = 0.0f32;
    let mut sidewards: f32 = 0.0f32;
    let mut turn: f32 = 0.0f32;

    'mainLoop: loop {
        let mut key_bytes = [0u8];
        stdin.read(&mut key_bytes).unwrap();

        match key_bytes[0] {
            b'x' => break 'mainLoop,
            b't' => drone.take_off().unwrap(),
            b'l' => drone.land().unwrap(),
            b'q' => turn = -1.0f32,
            b'e' => turn = 1.0f32,
            b'r' => upwards = 1.0f32,
            b'f' => upwards = -1.0f32,
            b'w' => forwards = 1.0f32,
            b's' => forwards = -1.0f32,
            b'd' => sidewards = 1.0f32,
            b'a' => sidewards = -1.0f32,
            b' ' => {
                turn = 0.0f32;
                forwards = 0.0f32;
                sidewards = 0.0f32;
                upwards = 0.0f32;
            }
            _ => (),
        };

        if let Some(msg) = drone.poll() {
            match msg {
                Message::Data(Package {
                    data: PackageData::FlightData(d),
                    ..
                }) => {
                    fl_data = Some(d);
                }
                Message::Data(Package {
                    data: PackageData::LightInfo(d),
                    ..
                }) => {
                    li_info = Some(d);
                }
                Message::Response(ResponseMsg::Connected(_)) => {
                    connected = true;
                }
                _ => (),
            }
        }

        drone
            .send_stick(upwards, sidewards, forwards, turn, false)
            .unwrap();

        write!(stdout, "{}", clear::All)?;
        write!(stdout, "{}DJI Tello CLI", cursor::Goto(1, 1))?;
        write!(stdout, "{}(x) Quit", cursor::Goto(1, 3))?;
        write!(stdout, "{}(t) Take off", cursor::Goto(1, 4))?;
        write!(stdout, "{}(l) Land now", cursor::Goto(1, 5))?;

        write!(
            stdout,
            "{}Connected {}",
            cursor::Goto(1, 12),
            if connected { "Yes" } else { "No" }
        )?;
        write!(
            stdout,
            "{}Battery {} %",
            cursor::Goto(17, 12),
            fl_data
                .as_ref()
                .and_then(|d| Some(d.battery_percentage))
                .unwrap_or(0)
        )?;
        write!(
            stdout,
            "{}Hight {} dm",
            cursor::Goto(35, 12),
            fl_data.as_ref().and_then(|d| Some(d.height)).unwrap_or(0)
        )?;
        write!(
            stdout,
            "{}FlightTime {} sec",
            cursor::Goto(50, 12),
            fl_data
                .as_ref()
                .and_then(|d| Some(d.fly_time as f32 / 10.0f32))
                .unwrap_or(0f32)
        )?;

        stdout.flush()?;

        sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }
    Ok(())
}
