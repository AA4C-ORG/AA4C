//! M0 实验（ADR 0005）：quinn + rustls 下两端用 Ed25519 原始公钥（RFC 7250）完成 QUIC 握手并互相校验公钥。
//!
//! 结论写回 ADR 0005；实验通过后，这里的做法移入 `aa4c-net`，本 crate 删除。

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use rustls::client::ResolvesClientCert;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::ring::default_provider;
use rustls::crypto::{
    CryptoProvider, WebPkiSupportedAlgorithms, verify_tls13_signature_with_raw_key,
};
use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, SubjectPublicKeyInfoDer,
    UnixTime,
};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::{DigitallySignedStruct, DistinguishedName, SignatureScheme};

pub type BoxError = Box<dyn Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

pub const ALPN: &[u8] = b"aa4c/1";

/// Ed25519 公钥（32 字节）。
pub type PublicKey = [u8; 32];

/// SubjectPublicKeyInfo 里 Ed25519 公钥前面固定的 12 字节（RFC 8410）。
const ED25519_SPKI_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

/// 从对方出示的 SPKI 里取出 Ed25519 公钥；任何其他算法或畸形编码都拒绝。
pub fn public_key_from_spki(spki: &[u8]) -> Option<PublicKey> {
    spki.strip_prefix(&ED25519_SPKI_PREFIX)?.try_into().ok()
}

fn spki_from_public_key(key: &PublicKey) -> Vec<u8> {
    let mut spki = ED25519_SPKI_PREFIX.to_vec();
    spki.extend_from_slice(key);
    spki
}

/// 一把设备身份：Ed25519 私钥（PKCS#8）与对应公钥。
pub struct Identity {
    certified: Arc<CertifiedKey>,
    public: PublicKey,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("public", &self.public)
            .finish_non_exhaustive()
    }
}

impl Identity {
    pub fn generate() -> Result<Self> {
        use ring::signature::{Ed25519KeyPair, KeyPair};
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())
            .map_err(|e| e.to_string())?;
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).map_err(|e| e.to_string())?;
        let public: PublicKey = pair
            .public_key()
            .as_ref()
            .try_into()
            .map_err(|_| "unexpected public key length")?;

        let key = default_provider()
            .key_provider
            .load_private_key(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                pkcs8.as_ref().to_vec(),
            )))?;
        // 原始公钥模式下，“证书链”的唯一元素就是 SPKI。
        let chain = vec![CertificateDer::from(spki_from_public_key(&public))];
        Ok(Self {
            certified: Arc::new(CertifiedKey::new(chain, key)),
            public,
        })
    }

    pub fn public_key(&self) -> PublicKey {
        self.public
    }
}

#[derive(Debug)]
struct RawKeyResolver(Arc<CertifiedKey>);

impl ResolvesClientCert for RawKeyResolver {
    fn resolve(&self, _: &[&[u8]], _: &[SignatureScheme]) -> Option<Arc<CertifiedKey>> {
        Some(self.0.clone())
    }

    fn only_raw_public_keys(&self) -> bool {
        true
    }

    fn has_certs(&self) -> bool {
        true
    }
}

impl ResolvesServerCert for RawKeyResolver {
    fn resolve(&self, _: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        Some(self.0.clone())
    }

    fn only_raw_public_keys(&self) -> bool {
        true
    }
}

/// 对端公钥的校验规则。
#[derive(Debug, Clone, Copy)]
pub enum Expect {
    /// 必须正好是这把公钥（已认证来源：连接卡片、二维码、邀请链接、已知设备证书）。
    Exactly(PublicKey),
    /// 先接受任何有效的 Ed25519 公钥，由应用在握手后经 [`peer_public_key`] 取出并当作 Stranger 处理。
    /// 只许监听方在“所有人”模式或持有有效口令时使用；拨号方永远不用。
    Any,
}

#[derive(Debug)]
struct PeerCheck {
    expect: Expect,
    algs: WebPkiSupportedAlgorithms,
}

impl PeerCheck {
    fn new(expect: Expect, provider: &CryptoProvider) -> Self {
        Self {
            expect,
            algs: provider.signature_verification_algorithms,
        }
    }

    fn check(&self, end_entity: &CertificateDer<'_>) -> std::result::Result<(), rustls::Error> {
        let bad = || {
            rustls::Error::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure,
            )
        };
        let key = public_key_from_spki(end_entity.as_ref()).ok_or_else(bad)?;
        match self.expect {
            Expect::Exactly(expected) if expected != key => Err(bad()),
            _ => Ok(()),
        }
    }

    fn verify_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        let spki = SubjectPublicKeyInfoDer::from(cert.as_ref().to_vec());
        verify_tls13_signature_with_raw_key(message, &spki, dss, &self.algs)
    }
}

fn tls12_unsupported() -> rustls::Error {
    rustls::Error::General("TLS 1.2 is not supported".into())
}

impl ServerCertVerifier for PeerCheck {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        self.check(end_entity)
            .map(|()| ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Err(tls12_unsupported())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        self.verify_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

impl ClientCertVerifier for PeerCheck {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: UnixTime,
    ) -> std::result::Result<ClientCertVerified, rustls::Error> {
        self.check(end_entity)
            .map(|()| ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Err(tls12_unsupported())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        self.verify_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(default_provider())
}

fn server_config(identity: &Identity, expect: Expect) -> Result<quinn::ServerConfig> {
    let provider = provider();
    let mut tls = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_client_cert_verifier(Arc::new(PeerCheck::new(expect, &provider)))
        .with_cert_resolver(Arc::new(RawKeyResolver(identity.certified.clone())));
    tls.alpn_protocols = vec![ALPN.to_vec()];
    Ok(quinn::ServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(tls)?,
    )))
}

fn client_config(identity: &Identity, expected_server: PublicKey) -> Result<quinn::ClientConfig> {
    let provider = provider();
    let mut tls = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(PeerCheck::new(
            Expect::Exactly(expected_server),
            &provider,
        )))
        .with_client_cert_resolver(Arc::new(RawKeyResolver(identity.certified.clone())));
    tls.alpn_protocols = vec![ALPN.to_vec()];
    Ok(quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(tls)?,
    )))
}

/// 监听端点。`expect` 决定接受谁的来电公钥。
pub fn listen(addr: SocketAddr, identity: &Identity, expect: Expect) -> Result<quinn::Endpoint> {
    Ok(quinn::Endpoint::server(
        server_config(identity, expect)?,
        addr,
    )?)
}

/// 拨号：必须给出预期的服务端公钥，握手时就校验，不存在“先接受再说”。
pub async fn dial(
    addr: SocketAddr,
    identity: &Identity,
    expected_server: PublicKey,
) -> Result<quinn::Connection> {
    let mut endpoint = quinn::Endpoint::client(SocketAddr::from(([0, 0, 0, 0], 0)))?;
    endpoint.set_default_client_config(client_config(identity, expected_server)?);
    // 原始公钥模式不使用服务器名；quinn 要求给一个。
    Ok(endpoint.connect(addr, "aa4c")?.await?)
}

/// 握手后取出对端经 TLS 认证的 Ed25519 公钥。
pub fn peer_public_key(connection: &quinn::Connection) -> Option<PublicKey> {
    let identity = connection.peer_identity()?;
    let chain = identity.downcast::<Vec<CertificateDer<'static>>>().ok()?;
    public_key_from_spki(chain.first()?.as_ref())
}
