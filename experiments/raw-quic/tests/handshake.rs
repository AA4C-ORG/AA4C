#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::net::SocketAddr;
use std::time::Duration;

use raw_quic::{Expect, Identity, dial, listen, peer_public_key, public_key_from_spki};

const LOOPBACK: &str = "127.0.0.1:0";
const WAIT: Duration = Duration::from_secs(10);

fn loopback() -> SocketAddr {
    LOOPBACK.parse().unwrap()
}

/// 服务端接受一个连接，读一条消息并回显；返回对端公钥。
async fn echo_once(endpoint: quinn::Endpoint) -> Option<[u8; 32]> {
    let connection = endpoint.accept().await?.await.ok()?;
    let peer = peer_public_key(&connection);
    let (mut send, mut recv) = connection.accept_bi().await.ok()?;
    let data = recv.read_to_end(1024).await.ok()?;
    send.write_all(&data).await.ok()?;
    send.finish().ok()?;
    let _ = send.stopped().await;
    peer
}

#[tokio::test]
async fn both_sides_authenticate_each_other_by_raw_public_key() {
    let server_id = Identity::generate().unwrap();
    let client_id = Identity::generate().unwrap();
    let endpoint = listen(
        loopback(),
        &server_id,
        Expect::Exactly(client_id.public_key()),
    )
    .unwrap();
    let addr = endpoint.local_addr().unwrap();
    let server = tokio::spawn(echo_once(endpoint));

    let connection = tokio::time::timeout(WAIT, dial(addr, &client_id, server_id.public_key()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(peer_public_key(&connection), Some(server_id.public_key()));

    let (mut send, mut recv) = connection.open_bi().await.unwrap();
    send.write_all(b"hello aa4c").await.unwrap();
    send.finish().unwrap();
    assert_eq!(recv.read_to_end(1024).await.unwrap(), b"hello aa4c");

    assert_eq!(server.await.unwrap(), Some(client_id.public_key()));
}

#[tokio::test]
async fn dialer_rejects_a_server_with_an_unexpected_key() {
    let real_server = Identity::generate().unwrap();
    let impostor_expectation = Identity::generate().unwrap().public_key();
    let client_id = Identity::generate().unwrap();
    let endpoint = listen(loopback(), &real_server, Expect::Any).unwrap();
    let addr = endpoint.local_addr().unwrap();
    tokio::spawn(echo_once(endpoint));

    let result = tokio::time::timeout(WAIT, dial(addr, &client_id, impostor_expectation))
        .await
        .unwrap();
    assert!(
        result.is_err(),
        "dialer must not accept an unexpected server key"
    );
}

#[tokio::test]
async fn listener_rejects_a_client_with_an_unexpected_key() {
    let server_id = Identity::generate().unwrap();
    let invited = Identity::generate().unwrap();
    let intruder = Identity::generate().unwrap();
    let endpoint = listen(
        loopback(),
        &server_id,
        Expect::Exactly(invited.public_key()),
    )
    .unwrap();
    let addr = endpoint.local_addr().unwrap();
    let server = tokio::spawn(echo_once(endpoint));

    // TLS 1.3 的客户端认证失败在客户端看来是握手后才收到的关闭，所以真正的判据是：
    // 服务端从未把它当作已认证的对端，且连接拿不到任何数据。
    let outcome = tokio::time::timeout(WAIT, async {
        let connection = dial(addr, &intruder, server_id.public_key()).await?;
        let (mut send, mut recv) = connection.open_bi().await?;
        send.write_all(b"let me in").await?;
        send.finish()?;
        Ok::<_, raw_quic::BoxError>(recv.read_to_end(1024).await?)
    })
    .await
    .unwrap();
    assert!(outcome.is_err(), "intruder must not get an echo");
    assert_eq!(server.await.unwrap(), None);
}

#[tokio::test]
async fn any_mode_exposes_the_unknown_peer_key_for_stranger_handling() {
    let server_id = Identity::generate().unwrap();
    let stranger = Identity::generate().unwrap();
    let endpoint = listen(loopback(), &server_id, Expect::Any).unwrap();
    let addr = endpoint.local_addr().unwrap();
    let server = tokio::spawn(echo_once(endpoint));

    let connection = tokio::time::timeout(WAIT, dial(addr, &stranger, server_id.public_key()))
        .await
        .unwrap()
        .unwrap();
    let (mut send, mut recv) = connection.open_bi().await.unwrap();
    send.write_all(b"hi").await.unwrap();
    send.finish().unwrap();
    assert_eq!(recv.read_to_end(1024).await.unwrap(), b"hi");

    // 监听方在握手后拿到的是真实的对端公钥，可据此判定关系（这里是 Stranger）。
    assert_eq!(server.await.unwrap(), Some(stranger.public_key()));
}

#[test]
fn spki_parsing_rejects_anything_but_ed25519() {
    let id = Identity::generate().unwrap();
    let mut spki = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    spki.extend_from_slice(&id.public_key());
    assert_eq!(public_key_from_spki(&spki), Some(id.public_key()));

    let mut wrong_oid = spki.clone();
    wrong_oid[8] = 0x71; // Ed448-ish OID byte
    assert_eq!(public_key_from_spki(&wrong_oid), None);
    assert_eq!(public_key_from_spki(&spki[..spki.len() - 1]), None);
    spki.push(0);
    assert_eq!(public_key_from_spki(&spki), None);
    assert_eq!(public_key_from_spki(&[]), None);
}
