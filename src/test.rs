use std::{collections::HashMap, sync::Arc};

use ethers_signers::LocalWallet;
use rand::thread_rng;
use http::Uri;
use livekit::{
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioSourceOptions, RtcAudioSource},
    },
    RoomOptions,
};

use crate::{
    signed_login::{signed_login, SignedLoginMeta},
    wallet::Wallet,
};

#[test]
fn test_no_video() {
    no_video();
}

pub fn no_video() {
    let wallet = Wallet {
        inner: Arc::new(Box::new(LocalWallet::new(&mut thread_rng()))),
    };
    let meta = SignedLoginMeta::new(
        true,
        Uri::try_from("https://worlds-content-server.decentraland.org/world/shibu.dcl.eth")
            .unwrap(),
    );

    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    let task = rt.spawn(async move {
        let login = signed_login(Uri::try_from("https://worlds-content-server.decentraland.org/get-comms-adapter/world-prd-shibu.dcl.eth").unwrap(), wallet, meta).await;
        let adapter = login.fixed_adapter.unwrap();
        let (protocol, remote_address) = adapter.split_once(':').unwrap();
        assert_eq!(protocol, "livekit");

        let url = Uri::try_from(remote_address).unwrap();
        let address = format!(
            "{}://{}{}",
            url.scheme_str().unwrap_or_default(),
            url.host().unwrap_or_default(),
            url.path()
        );
        let params = HashMap::<&str, &str>::from_iter(url.query().unwrap_or_default().split('&').flat_map(|par| {
            par.split_once('=')
        }));
        println!("{params:?}");
        let token = params.get("access_token").cloned().unwrap_or_default();

        println!("address: {address}");
        println!("token: {token}");

        let (_room, _network_rx) = livekit::prelude::Room::connect(&address, token, RoomOptions{ auto_subscribe: true, adaptive_stream: false, dynacast: false }).await.unwrap();
    });

    rt.block_on(task).unwrap();
}

#[test]
fn test_with_video() {
    with_video();
}

pub fn with_video() {
    // boilerplate
    let wallet = Wallet {
        inner: Arc::new(Box::new(LocalWallet::new(&mut thread_rng()))),
    };
    let meta = SignedLoginMeta::new(
        true,
        Uri::try_from("https://worlds-content-server.decentraland.org/world/shibu.dcl.eth")
            .unwrap(),
    );

    let rt = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    let task = rt.spawn(async move {
        // still boilerplate
        let login = signed_login(Uri::try_from("https://worlds-content-server.decentraland.org/get-comms-adapter/world-prd-shibu.dcl.eth").unwrap(), wallet, meta).await;
        let adapter = login.fixed_adapter.unwrap();
        let (protocol, remote_address) = adapter.split_once(':').unwrap();
        assert_eq!(protocol, "livekit");

        let url = Uri::try_from(remote_address).unwrap();
        let address = format!(
            "{}://{}{}",
            url.scheme_str().unwrap_or_default(),
            url.host().unwrap_or_default(),
            url.path()
        );
        let params = HashMap::<String, String>::from_iter(url.query().unwrap_or_default().split('&').flat_map(|par| {
            par.split_once('=')
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
        }));
        println!("{params:?}");
        let token = params.get("access_token").cloned().unwrap_or_default();

        // meat
        let (room, _network_rx) = livekit::prelude::Room::connect(&address, &token, RoomOptions{ auto_subscribe: true, adaptive_stream: false, dynacast: false }).await.unwrap();
        let native_source = NativeAudioSource::new(AudioSourceOptions{
            echo_cancellation: true,
            noise_suppression: true,
            auto_gain_control: true,
        });
        let mic_track = LocalTrack::Audio(LocalAudioTrack::create_audio_track("mic", RtcAudioSource::Native(native_source.clone())));
        room.local_participant().publish_track(mic_track, TrackPublishOptions{ source: TrackSource::Microphone, ..Default::default() }).await.unwrap();
    });

    rt.block_on(task).unwrap();
}
