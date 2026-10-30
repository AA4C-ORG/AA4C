//! ADR 0015 要求实验验证的性质：载荷无稳定标识（含真实条目数）、旧包不能重放、
//! 大小与延迟有上限、删除联系人后不再生成可识别的条目、解密失败不泄露联系人是否存在。
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::HashSet;
use std::net::UdpSocket;

use beacon_exp::{
    Card, DATAGRAM_LEN, DiscoveryKey, OsRandom, Receiver, Reject, SLOTS_PER_DATAGRAM, build_cycle,
};

fn key(n: u8) -> DiscoveryKey {
    DiscoveryKey::from_bytes([n; 32])
}

fn keys(n: usize) -> Vec<DiscoveryKey> {
    (0..n)
        .map(|i| DiscoveryKey::from_bytes(blake3_like(i as u64)))
        .collect()
}

fn blake3_like(seed: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = (seed.wrapping_mul(2654435761).wrapping_add(i as u64 * 40503) >> 3) as u8 ^ (i as u8);
    }
    out[0] = (seed & 0xff) as u8;
    out[1] = ((seed >> 8) & 0xff) as u8;
    out
}

fn card() -> Card {
    Card {
        device_key: [7; 32],
        port: 41234,
        cert_digest: [9; 16],
    }
}

const WINDOW: u32 = 1_000_000;

fn cycle(keys: &[DiscoveryKey], window: u32) -> Vec<Vec<u8>> {
    build_cycle(&mut OsRandom, keys, &card(), window, &mut 0).unwrap()
}

#[test]
fn every_datagram_has_the_same_length() {
    for n in [0, 1, 13, 14, 100] {
        for datagram in cycle(&keys(n), WINDOW) {
            assert_eq!(datagram.len(), DATAGRAM_LEN, "{n} contacts");
        }
    }
}

#[test]
fn datagram_count_reveals_only_a_tier_not_the_real_entry_count() {
    let per = SLOTS_PER_DATAGRAM;
    let cases = [
        (0, 1),
        (1, 1),
        (per, 1),
        (per + 1, 2),
        (2 * per, 2),
        (2 * per + 1, 4),
        (100, 8),
        (8 * per, 8),
        (8 * per + 1, 16),
        (16 * per, 16),
    ];
    for (contacts, expected) in cases {
        assert_eq!(
            cycle(&keys(contacts), WINDOW).len(),
            expected,
            "{contacts} contacts"
        );
    }
}

#[test]
fn a_hundred_contacts_fit_in_one_small_cycle() {
    let cycle = cycle(&keys(100), WINDOW);
    let bytes: usize = cycle.iter().map(Vec::len).sum();
    assert!(
        cycle.len() <= 8 && bytes <= 8 * 1200,
        "{} datagrams, {bytes} bytes",
        cycle.len()
    );
}

#[test]
fn more_contacts_than_a_cycle_holds_are_rotated_and_all_get_found_within_two_cycles() {
    let all = keys(300);
    let mut cursor = 0;
    let mut found = HashSet::new();
    for _ in 0..2 {
        for datagram in build_cycle(&mut OsRandom, &all, &card(), WINDOW, &mut cursor).unwrap() {
            for (i, k) in all.iter().enumerate() {
                let mut receiver = Receiver::new([1; 32]);
                receiver.add_key(i as u64, k.clone());
                if !receiver.open(WINDOW, &datagram).unwrap().is_empty() {
                    found.insert(i);
                }
            }
        }
    }
    assert_eq!(found.len(), 300);
}

#[test]
fn the_right_contact_finds_the_card_and_nobody_else_does() {
    let all = keys(20);
    let mut cursor = 0;
    let datagrams = build_cycle(&mut OsRandom, &all, &card(), WINDOW, &mut cursor).unwrap();

    let mut bob = Receiver::new([1; 32]);
    bob.add_key(7, all[3].clone());
    let found: Vec<_> = datagrams
        .iter()
        .flat_map(|d| bob.open(WINDOW, d).unwrap())
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, 7);
    assert_eq!(found[0].1, card());

    let mut stranger = Receiver::new([1; 32]);
    stranger.add_key(1, key(99));
    assert!(
        datagrams
            .iter()
            .all(|d| stranger.open(WINDOW, d).unwrap().is_empty())
    );
}

#[test]
fn failure_to_decrypt_looks_the_same_with_or_without_contacts() {
    let datagrams = cycle(&keys(5), WINDOW);
    let mut nobody = Receiver::new([1; 32]);
    let mut other = Receiver::new([1; 32]);
    other.add_key(1, key(200));
    for d in &datagrams {
        assert_eq!(nobody.open(WINDOW, d), Ok(vec![]));
        assert_eq!(other.open(WINDOW, d), Ok(vec![]));
    }
}

#[test]
fn no_byte_run_or_substring_is_stable_across_cycles() {
    let all = keys(10);
    let first = cycle(&all, WINDOW);
    let second = cycle(&all, WINDOW + 5);
    let eights = |frames: &[Vec<u8>]| -> HashSet<Vec<u8>> {
        frames
            .iter()
            .flat_map(|f| f.windows(8).map(<[u8]>::to_vec).collect::<Vec<_>>())
            .collect()
    };
    let (a, b) = (eights(&first), eights(&second));
    // The window field differs by design; everything else must look random.
    assert!(
        a.intersection(&b).count() == 0,
        "a stable 8-byte substring leaked"
    );
    for x in &first {
        for y in &second {
            let mut run = 0;
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                run = if p == q { run + 1 } else { 0 };
                assert!(run < 4 || i < 5, "4 equal bytes in a row at offset {i}");
            }
        }
    }
}

#[test]
fn stale_windows_are_rejected_and_the_skew_allowance_is_one_window() {
    let datagrams = cycle(&keys(1), WINDOW);
    let mut receiver = Receiver::new([1; 32]);
    receiver.add_key(0, key_zero());
    for now in [WINDOW + 2, WINDOW + 100, WINDOW.saturating_sub(2)] {
        assert_eq!(
            receiver.open(now, &datagrams[0]),
            Err(Reject::Stale),
            "now={now}"
        );
    }
    for now in [WINDOW - 1, WINDOW, WINDOW + 1] {
        let mut fresh = Receiver::new([1; 32]);
        fresh.add_key(0, key_zero());
        assert!(fresh.open(now, &datagrams[0]).is_ok(), "now={now}");
    }
}

fn key_zero() -> DiscoveryKey {
    keys(1).remove(0)
}

#[test]
fn an_exact_replay_is_rejected() {
    let datagrams = cycle(&keys(1), WINDOW);
    let mut receiver = Receiver::new([1; 32]);
    receiver.add_key(0, key_zero());
    assert_eq!(receiver.open(WINDOW, &datagrams[0]).unwrap().len(), 1);
    assert_eq!(receiver.open(WINDOW, &datagrams[0]), Err(Reject::Replay));
}

#[test]
fn flipping_any_single_byte_never_yields_a_different_card() {
    let all = keys(SLOTS_PER_DATAGRAM);
    let datagram = cycle(&all, WINDOW).remove(0);
    let genuine = card();
    for i in 0..datagram.len() {
        let mut tampered = datagram.clone();
        tampered[i] ^= 0x01;
        let mut receiver = Receiver::new([1; 32]);
        receiver.add_key(0, all[0].clone());
        if let Ok(found) = receiver.open(WINDOW, &tampered) {
            assert!(
                found.iter().all(|(_, c)| *c == genuine),
                "byte {i} produced a forged card"
            );
        }
    }
}

#[test]
fn malformed_input_is_rejected_not_crashed_on() {
    let mut receiver = Receiver::new([1; 32]);
    receiver.add_key(0, key(1));
    for bad in [
        vec![],
        vec![0u8; 1],
        vec![1u8; DATAGRAM_LEN - 1],
        vec![1u8; DATAGRAM_LEN + 1],
        vec![0u8; DATAGRAM_LEN],
        vec![255u8; DATAGRAM_LEN],
    ] {
        assert!(
            matches!(
                receiver.open(WINDOW, &bad),
                Err(Reject::Malformed | Reject::Stale)
            ),
            "len {}",
            bad.len()
        );
    }
}

#[test]
fn a_device_ignores_its_own_beacon() {
    let own = card().device_key;
    let datagrams = cycle(&[key(1)], WINDOW);
    let mut receiver = Receiver::new(own);
    receiver.add_key(0, key(1));
    assert!(
        datagrams
            .iter()
            .all(|d| receiver.open(WINDOW, d).unwrap().is_empty())
    );
}

#[test]
fn a_deleted_contact_stops_being_discoverable_from_either_side() {
    let (k_ab, k_ac) = (key(1), key(2));

    // A deletes B: A stops generating a slot under k_ab.
    let after_a_deletes_b = cycle(std::slice::from_ref(&k_ac), WINDOW);
    let mut b = Receiver::new([1; 32]);
    b.add_key(0, k_ab.clone());
    assert!(
        after_a_deletes_b
            .iter()
            .all(|d| b.open(WINDOW, d).unwrap().is_empty())
    );

    // B deletes A: B stops trying k_ab.
    let normal = cycle(&[k_ab.clone(), k_ac], WINDOW + 1);
    let mut b = Receiver::new([1; 32]);
    b.add_key(0, k_ab);
    b.remove_key(0);
    assert!(
        normal
            .iter()
            .all(|d| b.open(WINDOW + 1, d).unwrap().is_empty())
    );
}

#[test]
fn the_largest_datagram_survives_a_real_udp_socket_and_fits_the_ipv6_minimum_mtu() {
    let receiver_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    receiver_socket
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let sender_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let datagram = cycle(&[key(1)], WINDOW).remove(0);
    sender_socket
        .send_to(&datagram, receiver_socket.local_addr().unwrap())
        .unwrap();
    let mut buffer = vec![0u8; 4096];
    let (n, _) = receiver_socket.recv_from(&mut buffer).unwrap();
    let mut receiver = Receiver::new([1; 32]);
    receiver.add_key(0, key(1));
    assert_eq!(receiver.open(WINDOW, &buffer[..n]).unwrap().len(), 1);
}
