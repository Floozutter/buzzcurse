use rdev::{listen, EventType};
use buttplug::{
    client::{
        ButtplugClient, ButtplugClientEvent, ButtplugClientDeviceMessageType, 
        VibrateCommand,
    },
    server::ButtplugServerOptions,
};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use futures::{StreamExt, Stream};
use futures_timer::Delay;
use std::sync::{Arc, Mutex};
use std::{error::Error, time::Duration};

async fn handle_scanning(mut event_stream: impl Stream<Item = ButtplugClientEvent> + Unpin) {
    loop {
        match event_stream.next().await.unwrap() {
            ButtplugClientEvent::DeviceAdded(dev) => {
                tokio::spawn(async move {
                    println!("device added: {}", dev.name);
                });
            },
            ButtplugClientEvent::ScanningFinished => {
                println!("scanning finished signaled!");
                return;
            },
            ButtplugClientEvent::ServerDisconnect => {
                println!("server disconnected!");
            },
            _ => {
                println!("something happened!");
            },
        }
    };
}

async fn run() -> Result<(), Box<dyn Error>> {
    // connect Buttplug devices
    let client = ButtplugClient::new("buzzcurse buttplug client");
    let event_stream = client.event_stream();
    client.connect_in_process(&ButtplugServerOptions::default()).await?;
    client.start_scanning().await?;
    let scan_handler = tokio::spawn(handle_scanning(event_stream));
    println!("\nscanning for devices! press enter at any point to stop scanning and connect MIDI.");
    BufReader::new(io::stdin()).lines().next_line().await?;
    client.stop_scanning().await?;
    scan_handler.await?;
    // listen to mouse events
    let devices = client.devices();
    let speed_a = Arc::new(Mutex::new(0.0 as f64));
    let speed_b = speed_a.clone();
    let _held = ();  // TODO
    let _pos = ();  // TODO
    tokio::spawn(async move {
        listen(move |event| {
            *speed_a.lock().unwrap() += match event.event_type {
                EventType::MouseMove { x: _, y: _ } => 0.01,
                _ => 0.0,
            };
        }).unwrap();
    });
    tokio::spawn(async move {
        loop {
            let speed = {
                let mut speed = speed_b.lock().unwrap();
                let clamped = (*speed).max(0.0).min(1.0);
                *speed = (clamped - 0.25).max(0.0);
                clamped
            };
            println!(
                "vibration speed: {:.5}  [{:<5}]",
                speed, "=".repeat((speed * 5.0) as usize)
            );
            for dev in devices.clone() {
                tokio::spawn(async move {
                    if dev.allowed_messages.contains_key(&ButtplugClientDeviceMessageType::VibrateCmd) {
                        dev.vibrate(VibrateCommand::Speed(speed)).await.unwrap();
                    }
                });
            }
            Delay::new(Duration::from_millis(50)).await;
        }
    });
    println!("\nconnected mouse input to device output! press enter at any point to quit.");
    BufReader::new(io::stdin()).lines().next_line().await?;
    println!("stopping all devices and quitting...");
    client.stop_all_devices().await?;
    println!("bye-bye! >:3c");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {}!", err);
    }
}
