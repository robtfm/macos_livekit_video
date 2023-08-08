use std::{env, sync::Arc};

use livekit::{Room, RoomOptions, DataPacketKind};

fn main() {
    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    let task = rt.spawn(async move {
        let url = env::var("LIVEKIT_URL").expect("LIVEKIT_URL is not set");
        let token = env::var("LIVEKIT_TOKEN").expect("LIVEKIT_TOKEN is not set");

        let (room, mut rx) = Room::connect(&url, &token, RoomOptions::default())
            .await
            .unwrap();
        println!("Connected to room: {} - {}", room.name(), room.sid());

        room.local_participant()
            .publish_data(
                "Hello world".to_owned().into_bytes(),
                DataPacketKind::Reliable,
                Default::default(),
            )
            .await
            .unwrap();

        while let Some(msg) = rx.recv().await {
            println!("Event: {:?}", msg);
        }
    });

    rt.block_on(task).unwrap();
}
