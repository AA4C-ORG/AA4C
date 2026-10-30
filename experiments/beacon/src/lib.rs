//! M0 实验（ADR 0015）：默认“仅联系人”信标的帧格式。
//!
//! 数据报 = 头（版本 1B + 时间窗口 4B + 随机数 24B，明文）+ 13 个等长的槽。每个槽是
//! `标签 4B + AEAD(卡片 64B) + 认证标签 16B`：给联系人的槽用该联系人的发现密钥生成，其余是随机填充。
//! 槽位打乱，每个周期的数据报数量按 1/2/4/8/16 的档位填充，所以载荷里没有稳定标识，也看不出真实联系人数。
//! 结论写回 ADR 0015；通过后实现移入 `aa4c-net`，本 crate 删除。

use std::collections::HashSet;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};

pub const VERSION: u8 = 1;
/// 时间窗口长度（秒）。接收方接受当前窗口及前后各一个，旧包超出就是 `Stale`。
pub const WINDOW_SECS: u64 = 30;

const NONCE_LEN: usize = 24;
const HEADER_LEN: usize = 1 + 4 + NONCE_LEN;
const LABEL_LEN: usize = 4;
const CARD_LEN: usize = 64;
const TAG_LEN: usize = 16;
const SLOT_LEN: usize = LABEL_LEN + CARD_LEN + TAG_LEN;
pub const SLOTS_PER_DATAGRAM: usize = 13;
pub const DATAGRAM_LEN: usize = HEADER_LEN + SLOTS_PER_DATAGRAM * SLOT_LEN;
// 1280（IPv6 最小 MTU）- 40（IP）- 8（UDP）：数据报不依赖分片。
const _: () = assert!(DATAGRAM_LEN <= 1232);
/// 每个周期的数据报数量只取这些档位，旁观者只能看到档位。
pub const TIERS: [usize; 5] = [1, 2, 4, 8, 16];
const MAX_TIER: usize = 16;
/// 一个周期最多容纳的条目；超出的联系人按游标在后续周期轮转。
pub const MAX_ENTRIES: usize = MAX_TIER * SLOTS_PER_DATAGRAM;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomError;

pub trait Random {
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), RandomError>;
}

#[derive(Debug, Default)]
pub struct OsRandom;

impl Random for OsRandom {
    fn fill(&mut self, buf: &mut [u8]) -> Result<(), RandomError> {
        getrandom::fill(buf).map_err(|_| RandomError)
    }
}

/// 添加联系人时经已认证通道交换的 256 位随机密钥。
#[derive(Clone, PartialEq, Eq)]
pub struct DiscoveryKey([u8; 32]);

impl std::fmt::Debug for DiscoveryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DiscoveryKey(..)")
    }
}

impl DiscoveryKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    fn label_key(&self) -> [u8; 32] {
        blake3::derive_key("aa4c beacon v1 label", &self.0)
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(&blake3::derive_key("aa4c beacon v1 aead", &self.0).into())
    }
}

/// 最小连接卡片：设备公钥、端口、设备证书摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub device_key: [u8; 32],
    pub port: u16,
    pub cert_digest: [u8; 16],
}

impl Card {
    fn encode(&self) -> [u8; CARD_LEN] {
        let mut out = [0u8; CARD_LEN];
        out[..32].copy_from_slice(&self.device_key);
        out[32..34].copy_from_slice(&self.port.to_be_bytes());
        out[34..50].copy_from_slice(&self.cert_digest);
        out
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            device_key: bytes.get(..32)?.try_into().ok()?,
            port: u16::from_be_bytes(bytes.get(32..34)?.try_into().ok()?),
            cert_digest: bytes.get(34..50)?.try_into().ok()?,
        })
    }
}

fn label(key: &DiscoveryKey, window: u32, nonce: &[u8; NONCE_LEN]) -> [u8; LABEL_LEN] {
    let mut hasher = blake3::Hasher::new_keyed(&key.label_key());
    hasher.update(&window.to_be_bytes());
    hasher.update(nonce);
    let mut out = [0u8; LABEL_LEN];
    out.copy_from_slice(&hasher.finalize().as_bytes()[..LABEL_LEN]);
    out
}

fn header(window: u32, nonce: &[u8; NONCE_LEN]) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[0] = VERSION;
    out[1..5].copy_from_slice(&window.to_be_bytes());
    out[5..].copy_from_slice(nonce);
    out
}

fn shuffle<T>(random: &mut impl Random, items: &mut [T]) -> Result<(), RandomError> {
    for i in (1..items.len()).rev() {
        let mut bytes = [0u8; 4];
        random.fill(&mut bytes)?;
        let j = (u32::from_be_bytes(bytes) as usize) % (i + 1);
        items.swap(i, j);
    }
    Ok(())
}

fn tier_for(entries: usize) -> usize {
    let needed = entries.div_ceil(SLOTS_PER_DATAGRAM).max(1);
    TIERS
        .iter()
        .copied()
        .find(|t| *t >= needed)
        .unwrap_or(MAX_TIER)
}

/// 生成一个周期要发出的全部数据报。
///
/// `cursor` 在条目超过 [`MAX_ENTRIES`] 时轮转选择联系人，调用方在周期之间保留它。
pub fn build_cycle(
    random: &mut impl Random,
    keys: &[DiscoveryKey],
    card: &Card,
    window: u32,
    cursor: &mut usize,
) -> Result<Vec<Vec<u8>>, RandomError> {
    let take = keys.len().min(MAX_ENTRIES);
    let start = if keys.is_empty() {
        0
    } else {
        *cursor % keys.len()
    };
    let selected: Vec<&DiscoveryKey> = keys.iter().cycle().skip(start).take(take).collect();
    *cursor = if keys.is_empty() {
        0
    } else {
        (start + take) % keys.len()
    };

    let total_slots = tier_for(selected.len()) * SLOTS_PER_DATAGRAM;
    let mut plan: Vec<Option<&DiscoveryKey>> = selected.into_iter().map(Some).collect();
    plan.resize(total_slots, None);
    shuffle(random, &mut plan)?;

    let plaintext = card.encode();
    let mut datagrams = Vec::new();
    for group in plan.chunks(SLOTS_PER_DATAGRAM) {
        let mut nonce = [0u8; NONCE_LEN];
        random.fill(&mut nonce)?;
        let head = header(window, &nonce);
        let mut datagram = head.to_vec();
        for slot in group {
            let mut bytes = [0u8; SLOT_LEN];
            match slot {
                Some(key) => {
                    bytes[..LABEL_LEN].copy_from_slice(&label(key, window, &nonce));
                    let sealed = key
                        .cipher()
                        .encrypt(
                            XNonce::from_slice(&nonce),
                            Payload {
                                msg: &plaintext,
                                aad: &head,
                            },
                        )
                        .map_err(|_| RandomError)?;
                    bytes[LABEL_LEN..].copy_from_slice(&sealed);
                }
                None => random.fill(&mut bytes)?,
            }
            datagram.extend_from_slice(&bytes);
        }
        datagrams.push(datagram);
    }
    Ok(datagrams)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reject {
    Malformed,
    Stale,
    Replay,
}

struct Entry {
    id: u64,
    label_key: [u8; 32],
    cipher: XChaCha20Poly1305,
}

pub struct Receiver {
    own_device: [u8; 32],
    keys: Vec<Entry>,
    seen: HashSet<(u32, [u8; NONCE_LEN])>,
}

impl std::fmt::Debug for Receiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Receiver")
            .field("keys", &self.keys.len())
            .finish_non_exhaustive()
    }
}

impl Receiver {
    pub fn new(own_device: [u8; 32]) -> Self {
        Self {
            own_device,
            keys: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_key(&mut self, id: u64, key: DiscoveryKey) {
        self.keys.push(Entry {
            id,
            label_key: key.label_key(),
            cipher: key.cipher(),
        });
    }

    pub fn remove_key(&mut self, id: u64) {
        self.keys.retain(|e| e.id != id);
    }

    /// 解开一个数据报，返回能认证的所有卡片。对没有任何匹配的数据报一律返回 `Ok(vec![])`，
    /// 不区分“没有联系人”与“密钥不对”。
    pub fn open(&mut self, now_window: u32, datagram: &[u8]) -> Result<Vec<(u64, Card)>, Reject> {
        if datagram.len() != DATAGRAM_LEN || datagram.first() != Some(&VERSION) {
            return Err(Reject::Malformed);
        }
        let window = u32::from_be_bytes(
            datagram
                .get(1..5)
                .and_then(|b| b.try_into().ok())
                .ok_or(Reject::Malformed)?,
        );
        if window.abs_diff(now_window) > 1 {
            return Err(Reject::Stale);
        }
        let head = datagram.get(..HEADER_LEN).ok_or(Reject::Malformed)?;
        let nonce: [u8; NONCE_LEN] = head
            .get(5..)
            .and_then(|b| b.try_into().ok())
            .ok_or(Reject::Malformed)?;
        let body = datagram.get(HEADER_LEN..).ok_or(Reject::Malformed)?;

        let mut found = Vec::new();
        for entry in &self.keys {
            let mut hasher = blake3::Hasher::new_keyed(&entry.label_key);
            hasher.update(&window.to_be_bytes());
            hasher.update(&nonce);
            let expected = hasher.finalize();
            let expected = expected.as_bytes().get(..LABEL_LEN).unwrap_or_default();
            for slot in body.chunks_exact(SLOT_LEN) {
                let (slot_label, sealed) = slot.split_at(LABEL_LEN);
                if slot_label != expected {
                    continue;
                }
                let Ok(plain) = entry.cipher.decrypt(
                    XNonce::from_slice(&nonce),
                    Payload {
                        msg: sealed,
                        aad: head,
                    },
                ) else {
                    continue;
                };
                if let Some(card) = Card::decode(&plain)
                    && card.device_key != self.own_device
                {
                    found.push((entry.id, card));
                }
            }
        }
        if found.is_empty() {
            return Ok(found);
        }
        // 只为真正解开的数据报记录随机数，攻击者灌入随机数据报撑不大缓存。
        self.seen.retain(|(w, _)| w.abs_diff(now_window) <= 1);
        if !self.seen.insert((window, nonce)) {
            return Err(Reject::Replay);
        }
        Ok(found)
    }
}
