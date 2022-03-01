extern crate evdev_rs as evdev;
extern crate mio;

use evdev::*;
use evdev::enums::*;
use std::io;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::os::unix::io::AsRawFd;
use mio::{Poll,Events,Token,Interest};
use mio::unix::SourceFd;

static BRIGHT_UP:   EventCode = EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY2);
static BRIGHT_DOWN: EventCode = EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY1);
static VOL_UP:      EventCode = EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY6);
static VOL_DOWN:    EventCode = EventCode::EV_KEY(EV_KEY::BTN_TRIGGER_HAPPY5);

fn process_event(_dev: &Device, ev: &InputEvent) {
    if ev.value == 1 {
        if ev.event_code == BRIGHT_UP {
            Command::new("brightness.sh").args(&["up"]).output().expect("Failed to execute brightness.sh");
        } else if ev.event_code == BRIGHT_DOWN {
            Command::new("brightness.sh").args(&["down"]).output().expect("Failed to execute brightness.sh");
        } else if ev.event_code == VOL_UP {
            Command::new("volume.sh").args(&["up"]).output().expect("Failed to execute volume.sh");
        } else if ev.event_code == VOL_DOWN {
            Command::new("volume.sh").args(&["down"]).output().expect("Failed to execute volume.sh");
        }
    } else if ev.event_code == EventCode::EV_SW(EV_SW::SW_HEADPHONE_INSERT) {
        let dest = match ev.value { 1 => "speaker", _ => "headphone" };
        Command::new("headphone.sh").args(&[dest]).output().expect("Failed to execute headphone.sh");
    }
}

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1);
    let mut devs: Vec<Device> = Vec::new();

    let mut i = 0;
    for s in ["/dev/input/event1", "/dev/input/event2"].iter() {
        if !Path::new(s).exists() {
            println!("Path {} doesn't exist", s);
            continue;
        }
        let fd = File::open(Path::new(s)).unwrap();
        let mut dev = Device::new().unwrap();
        poll.registry().register(&mut SourceFd(&fd.as_raw_fd()), Token(i), Interest::READABLE)?;
        dev.set_fd(fd)?;
        devs.push(dev);
        println!("Added {}", s);
        i += 1;
    }

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            let dev = &mut devs[event.token().0];
            while dev.has_event_pending() {
                let e = dev.next_event(evdev_rs::ReadFlag::NORMAL);
                match e {
                    Ok(k) => {
                        let ev = &k.1;
                        process_event(&dev, &ev);
                    },
                    _ => ()
                }
            }
        }
    }
}
