use surf::StatusCode;
use http::Uri;
use crate::wallet::{SimpleAuthChain, Wallet};

#[derive(Debug, serde::Deserialize)]
pub struct SignedLoginResponse {
    pub message: Option<String>,
    #[serde(rename = "fixedAdapter")]
    pub fixed_adapter: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SignedLoginMeta {
    pub intent: String,
    pub signer: String,
    #[serde(rename = "isGuest")]
    is_guest: bool,
    origin: String,
}

impl SignedLoginMeta {
    pub fn new(is_guest: bool, origin: Uri) -> Self {
        let origin = origin.into_parts();

        Self {
            intent: "dcl:explorer:comms-handshake".to_owned(),
            signer: "dcl:explorer".to_owned(),
            is_guest,
            origin: format!("{}://{}", origin.scheme.unwrap(), origin.authority.unwrap()),
        }
    }
}

pub async fn signed_login(
    uri: Uri,
    wallet: Wallet,
    meta: SignedLoginMeta,
) -> SignedLoginResponse {
    let unix_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let meta = serde_json::to_string(&meta).unwrap();

    let payload = format!("post:{}:{}:{}", uri.path(), unix_time, meta).to_lowercase();
    let signature = wallet.sign_message(&payload).await.unwrap();
    let auth_chain = SimpleAuthChain::new(wallet.address(), payload, signature);

    let mut builder = surf::post(uri.to_string());

    for (key, value) in auth_chain.headers() {
        builder = builder.header(key.as_str(), value)
    }

    let req = builder
        .header("x-identity-timestamp", format!("{unix_time}"))
        .header("x-identity-metadata", meta);

    let mut res = req.await.unwrap();

    if res.status() != StatusCode::Ok {
        panic!();
    }

    res.body_json().await.unwrap()
}
