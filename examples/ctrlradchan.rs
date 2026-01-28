use std::{fs, time::Duration};

use vex_v5_serial::{
    Connection,
    protocol::{
        cdc2::controller::{
            ConFlashReadPacket, ConFlashReadPayload,
        },
    },
    serial::{self, SerialError},
};

fn translate_to_vexese(mut offset: u32) -> (u8, u8, u8) {
    let KB = offset / 0x10000;
    offset -= KB * 0x10000;
    let sectors = offset / 0x100;
    offset -= sectors;
    let bytes = offset;
    (bytes as u8, sectors as u8, KB as u8)
}

#[tokio::main]
async fn main() -> Result<(), SerialError> {
    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Always,
    )
    .unwrap();

    let devices = serial::find_devices()?;

    // Open a connection to the device
    let mut connection = devices[0].connect(Duration::from_secs(30))?;

    let mut file: Vec<u8> = Vec::new();

    

    for n in 0..8*256 {
        let offset =  0x10000000-3 + n*128;
        let (bytes,sectors,kb) = translate_to_vexese(offset);
        let _ = connection
            .send(ConFlashReadPacket::new(ConFlashReadPayload {
                asset: vex_v5_serial::protocol::cdc2::controller::ConFlashAssetIdx::FlashBase,
                offset_bytes: bytes,
                offset_sectors: sectors,
                offset_65kb: kb,
                size: 128,
            }))
            .await;
        let status = connection
            .recv::<[u8; 128 + 6]>(Duration::from_millis(500))
            .await
            .unwrap();
        file.extend_from_slice(&status[6..]);
    }

    fs::write("ramdump.bin", file);
    Ok(())
}
