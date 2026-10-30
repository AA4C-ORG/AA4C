//! 测量：`cargo run --release -p beacon-exp --example measure`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout
)]

use std::time::Instant;

use beacon_exp::{
    Card, DATAGRAM_LEN, DiscoveryKey, MAX_ENTRIES, OsRandom, Receiver, SLOTS_PER_DATAGRAM,
    build_cycle,
};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

fn raw_keys(n: usize) -> Vec<[u8; 32]> {
    (0..n)
        .map(|i| {
            let mut b = [0u8; 32];
            b[..8].copy_from_slice(&(i as u64 + 1).to_be_bytes());
            b[8..16].copy_from_slice(&(i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15).to_be_bytes());
            b
        })
        .collect()
}

fn keys(n: usize) -> Vec<DiscoveryKey> {
    raw_keys(n)
        .into_iter()
        .map(DiscoveryKey::from_bytes)
        .collect()
}

fn main() {
    let card = Card {
        device_key: [7; 32],
        port: 41234,
        cert_digest: [9; 16],
    };
    const PERIOD_S: f64 = 2.0;
    println!(
        "datagram = {DATAGRAM_LEN} B, {SLOTS_PER_DATAGRAM} slots, max {MAX_ENTRIES} entries per cycle, period {PERIOD_S}s\n"
    );
    println!(
        "{:>9} {:>10} {:>10} {:>12} {:>12}",
        "contacts", "datagrams", "bytes", "bytes/s", "cycles for all"
    );
    for n in [0usize, 1, 13, 50, 100, 208, 300, 1000] {
        let all = keys(n);
        let mut cursor = 0;
        let cycle = build_cycle(&mut OsRandom, &all, &card, 1, &mut cursor).unwrap();
        let bytes: usize = cycle.iter().map(Vec::len).sum();
        let cycles_for_all = n.div_ceil(MAX_ENTRIES).max(1);
        println!(
            "{n:>9} {:>10} {bytes:>10} {:>12.0} {cycles_for_all:>12}",
            cycle.len(),
            bytes as f64 / PERIOD_S
        );
    }

    // 接收方 CPU：一个邻居的一个周期（100 个联系人 -> 8 个数据报），接收方持有 M 把密钥。
    println!("\nreceiver cost for one neighbour's cycle of 8 datagrams");
    let sender = keys(100);
    let mut cursor = 0;
    let cycle = build_cycle(&mut OsRandom, &sender, &card, 1, &mut cursor).unwrap();
    for m in [10usize, 100, 500] {
        let mut receiver = Receiver::new([1; 32]);
        for (i, k) in keys(m).into_iter().enumerate() {
            receiver.add_key(i as u64, k);
        }
        let reps = 200;
        let start = Instant::now();
        for _ in 0..reps {
            for d in &cycle {
                let _ = receiver.open(1, d); // 重复喂同一周期会得到 Replay，解密工作量不变
            }
        }
        let per_cycle = start.elapsed().as_secs_f64() / f64::from(reps);
        println!(
            "  labels:      {m:>4} keys -> {:>8.1} us/cycle",
            per_cycle * 1e6
        );

        // 对照：没有标签，对每个槽、每把密钥都试解密。
        let ciphers: Vec<XChaCha20Poly1305> = raw_keys(m)
            .iter()
            .map(|k| XChaCha20Poly1305::new(&blake3::derive_key("aa4c beacon v1 aead", k).into()))
            .collect();
        let start = Instant::now();
        for _ in 0..reps {
            for d in &cycle {
                let nonce = &d[5..29];
                for slot in d[29..].as_chunks::<84>().0 {
                    for c in &ciphers {
                        let _ = c.decrypt(
                            XNonce::from_slice(nonce),
                            Payload {
                                msg: &slot[4..],
                                aad: &d[..29],
                            },
                        );
                    }
                }
            }
        }
        let per_cycle = start.elapsed().as_secs_f64() / f64::from(reps);
        println!(
            "  trial-only:  {m:>4} keys -> {:>8.1} us/cycle",
            per_cycle * 1e6
        );
    }

    // 发现延迟：每周期每个槽发一次，数据报在周期内均匀发出；丢包率 p。
    println!("\ndiscovery latency, period {PERIOD_S}s, datagrams spread evenly over the period");
    for loss in [0.0f64, 0.1, 0.3] {
        let mut samples = Vec::new();
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..20_000 {
            // 接收方在一个周期内的随机时刻开始监听；我的槽在数据报 k 里，发出时刻 k/8*P。
            let listen_at = next() * PERIOD_S;
            let slot_at = next() * PERIOD_S;
            let mut t = if slot_at >= listen_at {
                slot_at - listen_at
            } else {
                slot_at + PERIOD_S - listen_at
            };
            while next() < loss {
                t += PERIOD_S;
            }
            samples.push(t);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p = |q: f64| samples[((samples.len() as f64 * q) as usize).min(samples.len() - 1)];
        println!(
            "  loss {:>3.0}%: median {:>5.2}s  p99 {:>5.2}s  max {:>5.2}s",
            loss * 100.0,
            p(0.5),
            p(0.99),
            samples[samples.len() - 1]
        );
    }
}
