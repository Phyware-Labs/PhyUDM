//! Merkle trees.
//!
//! [RFC 9162](https://www.rfc-editor.org/rfc/rfc9162#section-2.1) trees are
//! used for batch signatures (spec `provenance.md`) and for evidence
//! manifests from version 3 (spec `evidence-package.md`).
//! [`legacy_sorted_pairs_root`] is the tree of version 2 manifests, kept so
//! that existing manifests still verify.

use sha2::{Digest, Sha256};

use crate::hex::{self, Hash};

/// Hash of a leaf: `SHA-256(0x00 ‖ data)`.
pub fn leaf_hash(data: &[u8]) -> Hash {
    let mut h = Sha256::new();
    h.update([0u8]);
    h.update(data);
    h.finalize().into()
}

/// Hash of an interior node: `SHA-256(0x01 ‖ left ‖ right)`.
pub fn node_hash(left: &Hash, right: &Hash) -> Hash {
    let mut h = Sha256::new();
    h.update([1u8]);
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// The largest power of two smaller than `n` (`n > 1`).
const fn split(n: usize) -> usize {
    let mut k = 1;
    while k * 2 < n {
        k *= 2;
    }
    k
}

/// RFC 9162 Merkle tree hash of leaf hashes; `SHA-256("")` for none.
pub fn merkle_root(leaves: &[Hash]) -> Hash {
    match leaves {
        [] => Sha256::digest([]).into(),
        [only] => *only,
        _ => {
            let (left, right) = leaves.split_at(split(leaves.len()));
            node_hash(&merkle_root(left), &merkle_root(right))
        }
    }
}

/// RFC 9162 inclusion proof for `leaves[index]`, leaf first; `None` if
/// `index` is out of range.
pub fn audit_path(index: usize, leaves: &[Hash]) -> Option<Vec<Hash>> {
    if index >= leaves.len() {
        return None;
    }
    if leaves.len() == 1 {
        return Some(Vec::new());
    }
    let (left, right) = leaves.split_at(split(leaves.len()));
    let (mut path, sibling) = if index < left.len() {
        (audit_path(index, left)?, merkle_root(right))
    } else {
        (audit_path(index - left.len(), right)?, merkle_root(left))
    };
    path.push(sibling);
    Some(path)
}

/// The root an inclusion proof leads to (RFC 9162 §2.1.3.2), or `None` if
/// the proof does not fit a tree of `size` leaves.
pub fn root_from_proof(index: u64, size: u64, leaf: &Hash, proof: &[Hash]) -> Option<Hash> {
    if index >= size {
        return None;
    }
    let (mut f, mut s, mut r) = (index, size - 1, *leaf);
    for p in proof {
        if s == 0 {
            return None;
        }
        if f & 1 == 1 || f == s {
            r = node_hash(p, &r);
            if f & 1 == 0 {
                while f != 0 && f & 1 == 0 {
                    f >>= 1;
                    s >>= 1;
                }
            }
        } else {
            r = node_hash(&r, p);
        }
        f >>= 1;
        s >>= 1;
    }
    (s == 0).then_some(r)
}

/// Root of a version 2 evidence manifest (`merkle_algorithm`
/// `sha256-sorted-pairs`), over lowercase hex leaf strings: each pair of
/// hex strings is sorted, concatenated and hashed (SHA-256 of the
/// concatenated text, as hex), an odd level repeats its last node, and no
/// leaves give `SHA-256("")`.
///
/// Version 3 manifests use [`merkle_root`] instead, which also fixes the
/// order of events. Verifiers report a version 2 root as legacy.
pub fn legacy_sorted_pairs_root(leaves: &[String]) -> String {
    let sha = |s: &str| hex::encode(&Sha256::digest(s.as_bytes()));
    if leaves.is_empty() {
        return sha("");
    }
    let mut nodes: Vec<String> = leaves.to_vec();
    while nodes.len() > 1 {
        if nodes.len() % 2 != 0 {
            if let Some(last) = nodes.last().cloned() {
                nodes.push(last);
            }
        }
        nodes = nodes
            .chunks_exact(2)
            .map(|pair| {
                let (a, b) = (&pair[0], &pair[1]);
                if a <= b {
                    sha(&format!("{a}{b}"))
                } else {
                    sha(&format!("{b}{a}"))
                }
            })
            .collect();
    }
    nodes.into_iter().next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_root_matches_known_shape() {
        let h = |s: &str| hex::encode(&Sha256::digest(s.as_bytes()));
        assert_eq!(legacy_sorted_pairs_root(&[]), h(""));
        let a = h("a");
        let b = h("b");
        assert_eq!(legacy_sorted_pairs_root(std::slice::from_ref(&a)), a);
        let (lo, hi) = if a <= b { (&a, &b) } else { (&b, &a) };
        assert_eq!(
            legacy_sorted_pairs_root(&[a.clone(), b.clone()]),
            h(&format!("{lo}{hi}"))
        );
    }

    #[test]
    fn proofs_round_trip() {
        let leaves: Vec<Hash> = (0u8..7).map(|i| leaf_hash(&[i])).collect();
        let root = merkle_root(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = audit_path(i, &leaves).unwrap();
            assert_eq!(root_from_proof(i as u64, 7, leaf, &proof), Some(root));
            let other = leaf_hash(b"other");
            assert_ne!(root_from_proof(i as u64, 7, &other, &proof), Some(root));
        }
    }
}
