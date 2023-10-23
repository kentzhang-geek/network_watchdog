use std::thread::sleep;
use std::net::IpAddr;
use winping::{Buffer, Pinger};
use std::process::Command;
extern crate ini;
use ini::Ini;

fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

fn write_sample_ini() {
    let mut conf = Ini::new();
    conf.with_section(None::<String>)
        .set("network_name", "Ethernet");
    conf.with_section(None::<String>)
        .set("sleep_sec", "300");
    conf.with_section(None::<String>)
        .set("wait_restart", "3");
    conf.with_section(None::<String>)
        .set("debug", "0");
    conf.write_to_file("conf.ini").unwrap();
}

#[windows_subsystem = "windows"]
fn main() {
    // if conf file not exit
    if !std::path::Path::new("conf.ini").exists() {
        write_sample_ini();
    }

    // read conf file
    let conf = Ini::load_from_file("conf.ini").unwrap();
    let network_name = conf.section(None::<String>).unwrap().get("network_name").unwrap();
    println!("network_name: {}", network_name);
    // sleep_sec to u32
    let sleep_sec = conf.section(None::<String>).unwrap().get("sleep_sec").unwrap().parse::<u32>().unwrap();
    println!("sleep_sec: {}", sleep_sec);
    let restart_wait = conf.section(None::<String>).unwrap().get("wait_restart").unwrap().parse::<u32>().unwrap();
    println!("wait_restart: {}", restart_wait);
    let is_debug = conf.section(None::<String>).unwrap().get("debug").unwrap().parse::<u32>().unwrap();
    println!("debug: {}", is_debug);
    if (is_debug == 1) {
    } else{
        hide_console_window();
    }

    let dst = std::env::args()
        .nth(1)
        .unwrap_or(String::from("8.8.8.8"))
        .parse::<IpAddr>()
        .expect("Could not parse IP Address");

    let pinger = Pinger::new().unwrap();
    let mut buffer = Buffer::new();

    loop {
        match pinger.send(dst, &mut buffer) {
            Ok(rtt) => println!("Response time {} ms.", rtt),
            Err(err) => {
                println!("{}.", err);
                Command::new("netsh")
                    .args(&["interface", "set", "interface", network_name, "DISABLED"])
                    .output()
                    .expect("Failed to execute command");
                println!("Disabled network interface {}", network_name);
                sleep(std::time::Duration::from_secs(restart_wait.into()));
                Command::new("netsh")
                    .args(&["interface", "set", "interface", network_name, "ENABLED"])
                    .output()
                    .expect("Failed to execute command");
                println!("Enabled network interface {}", network_name);
            },
        }
        sleep(std::time::Duration::from_secs(sleep_sec.into()));
    }
}
