/// SRP6a as used by HAP (HomeKit Accessory Protocol / AirPlay 2 PAIR-SETUP).
///
/// Group: 3072-bit prime from RFC 3526 §7, g = 5.
/// Hash: SHA-512.
/// Username (I): "Pair-Setup"
/// Password (P): 8-digit PIN string, e.g. "12345678"

use num_bigint::BigUint;
use num_traits::Zero;
use sha2::{Digest, Sha512};

// RFC 3526 3072-bit prime
const N_HEX: &str = concat!(
    "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD1",
    "29024E088A67CC74020BBEA63B139B22514A08798E3404DD",
    "EF9519B3CD3A431B302B0A6DF25F14374FE1356D6D51C245",
    "E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7ED",
    "EE386BFB5A899FA5AE9F24117C4B1FE649286651ECE45B3D",
    "C2007CB8A163BF0598DA48361C55D39A69163FA8FD24CF5F",
    "83655D23DCA3AD961C62F356208552BB9ED529077096966D",
    "670C354E4ABC9804F1746C08CA18217C32905E462E36CE3B",
    "E39E772C180E86039B2783A2EC07A28FB5C55DF06F4C52C9",
    "DE2BCBF6955817183995497CEA956AE515D2261898FA0510",
    "15728E5A8AAAC42DAD33170D04507A33A85521ABDF1CBA64",
    "ECFB850458DBEF0A8AEA71575D060C7DB3970F85A6E1E4C7",
    "ABF5AE8CDB0933D71E8CE9396A2C4F8C6F8D78D6E3BE6A2E",
    "F41A7D2CF2F4C3A1B7BC6BBEB12E7A9E01C49B85A6A9D3AF",
    "F0517867B7EC5D3A8A01B2FE46DC2EF7C18C0A9B7D8EBEA2",
    "F3B50B8A1EB0ECF5D58B64B1E9B3DAD9D8AD8C0B46BBBDCE",
    "2DEE6E37",
);
const G: u64 = 5;

pub struct SrpClient {
    n: BigUint,
    g: BigUint,
    /// Client private key (random 256-bit)
    a: BigUint,
    /// Client public key A = g^a mod N
    pub a_pub: Vec<u8>,
}

impl SrpClient {
    pub fn new() -> Self {
        let n = BigUint::parse_bytes(N_HEX.as_bytes(), 16).unwrap();
        let g = BigUint::from(G);

        // Random 256-bit private key
        let a_bytes: [u8; 32] = rand::random();
        let a = BigUint::from_bytes_be(&a_bytes);
        let a_pub = g.modpow(&a, &n).to_bytes_be();

        Self { n, g, a, a_pub }
    }

    /// Given salt (s) and server public key (B), compute the client proof M1
    /// and the shared session key K. Returns (M1, K).
    pub fn compute(
        &self,
        username: &str,
        password: &str,
        salt: &[u8],
        b_pub: &[u8],
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        let n = &self.n;
        let g = &self.g;
        let n_len = (n.bits() as usize + 7) / 8;

        let b = BigUint::from_bytes_be(b_pub);
        // B must be non-zero and < N
        if b.is_zero() || b >= *n {
            return None;
        }

        // k = H(N | PAD(g))
        let k = {
            let mut h = Sha512::new();
            h.update(pad(n.to_bytes_be(), n_len));
            h.update(pad(g.to_bytes_be(), n_len));
            BigUint::from_bytes_be(&h.finalize())
        };

        // x = H(s | H(I ":" P))
        let x = {
            let mut h1 = Sha512::new();
            h1.update(username.as_bytes());
            h1.update(b":");
            h1.update(password.as_bytes());
            let ih = h1.finalize();
            let mut h2 = Sha512::new();
            h2.update(salt);
            h2.update(&ih);
            BigUint::from_bytes_be(&h2.finalize())
        };

        // u = H(PAD(A) | PAD(B))
        let u = {
            let mut h = Sha512::new();
            h.update(pad(self.a_pub.clone(), n_len));
            h.update(pad(b_pub.to_vec(), n_len));
            BigUint::from_bytes_be(&h.finalize())
        };

        // S = (B - k*v)^(a + u*x) mod N  where v = g^x mod N
        let v = g.modpow(&x, n);
        let kv = (k * &v) % n;
        let base = if b >= kv { (b - kv) % n } else { (b + n - kv % n) % n };
        let exp = (&self.a + &u * &x) % (n - BigUint::from(1u32));
        let s_val = base.modpow(&exp, n);

        let s_bytes = pad(s_val.to_bytes_be(), n_len);

        // K = H(S)
        let k_session: Vec<u8> = Sha512::digest(&s_bytes).to_vec();

        // M1 = H(H(N) XOR H(g) | H(I) | s | A | B | K)
        let h_n: Vec<u8> = Sha512::digest(&pad(n.to_bytes_be(), n_len)).to_vec();
        let h_g: Vec<u8> = Sha512::digest(&pad(g.to_bytes_be(), n_len)).to_vec();
        let h_ng: Vec<u8> = h_n.iter().zip(h_g.iter()).map(|(a, b)| a ^ b).collect();
        let h_i: Vec<u8> = Sha512::digest(username.as_bytes()).to_vec();

        let m1 = {
            let mut h = Sha512::new();
            h.update(&h_ng);
            h.update(&h_i);
            h.update(salt);
            h.update(pad(self.a_pub.clone(), n_len));
            h.update(pad(b_pub.to_vec(), n_len));
            h.update(&k_session);
            h.finalize().to_vec()
        };

        Some((m1, k_session))
    }

    /// Verify the server's proof M2 = H(A | M1 | K).
    pub fn verify_server(a_pub: &[u8], m1: &[u8], k: &[u8], m2_server: &[u8]) -> bool {
        let expected = Sha512::digest(
            [a_pub, m1, k].concat()
        );
        expected.as_slice() == m2_server
    }
}

fn pad(mut v: Vec<u8>, len: usize) -> Vec<u8> {
    if v.len() < len {
        let mut padded = vec![0u8; len - v.len()];
        padded.append(&mut v);
        padded
    } else {
        v
    }
}
