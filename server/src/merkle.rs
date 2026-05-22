use anyhow::Context;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::state::AppState;

// Snapshot layout for the $ZePIN claim distributor on Solana — deliberately
// Solana-friendly so the on-chain program can verify a Merkle proof without
// any tricks:
//
//   leaf  = sha256( base58_wallet || u64_le(points) )
//   node  = sha256( sort(left, right) )                  (sorted-pair so proofs work without index)
//
// All hashes are 32 bytes, hex-encoded for storage / JSON.

#[derive(Debug)]
pub struct PublishResult {
    pub cycle: i64,
    pub merkle_root: String,
    pub leaves: usize,
    pub total_points: u64,
}

pub async fn publish_snapshot(state: &AppState) -> anyhow::Result<PublishResult> {
    let cfg = state.config();
    let mut leaves = state
        .store()
        .total_points_per_wallet(cfg.network.as_str())
        .await
        .context("loading per-wallet point totals")?;
    leaves.sort_by(|a, b| a.0.cmp(&b.0));

    if leaves.is_empty() {
        anyhow::bail!("no eligible wallets — cannot publish empty snapshot");
    }

    let total_points: u64 = leaves.iter().map(|(_, p)| *p).sum();
    let leaf_hashes: Vec<[u8; 32]> = leaves
        .iter()
        .map(|(wallet, pts)| hash_leaf(wallet, *pts))
        .collect();

    let tree = build_tree(&leaf_hashes);
    let root_hex = hex::encode(tree.root);

    // Pick next cycle number = max(existing) + 1 (1-indexed).
    let last_cycle = match state.store().latest_snapshot().await? {
        Some((_, cycle, _, _)) => cycle,
        None => 0,
    };
    let cycle = last_cycle + 1;
    let snapshot_id = state
        .store()
        .insert_snapshot(cycle, &root_hex, total_points, cfg.spl_mint.as_deref())
        .await?;

    for (idx, (wallet, points)) in leaves.iter().enumerate() {
        let leaf_hash = leaf_hashes[idx];
        let proof = tree.proof_for(idx);
        let proof_json = serde_json::json!({
            "siblings": proof.iter().map(hex::encode).collect::<Vec<_>>(),
            "leaf_index": idx,
        });
        state
            .store()
            .insert_snapshot_leaf(
                snapshot_id,
                wallet,
                *points,
                &hex::encode(leaf_hash),
                &serde_json::to_string(&proof_json)?,
            )
            .await?;
    }

    tracing::info!(
        cycle,
        leaves = leaves.len(),
        total_points,
        merkle_root = %root_hex,
        published_at = %Utc::now(),
        "snapshot published"
    );

    Ok(PublishResult {
        cycle,
        merkle_root: root_hex,
        leaves: leaves.len(),
        total_points,
    })
}

pub fn hash_leaf(wallet: &str, points: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(wallet.as_bytes());
    hasher.update(points.to_le_bytes());
    hasher.finalize().into()
}

fn hash_pair_sorted(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let mut hasher = Sha256::new();
    hasher.update(lo);
    hasher.update(hi);
    hasher.finalize().into()
}

struct MerkleTree {
    layers: Vec<Vec<[u8; 32]>>, // layer 0 = leaves
    root: [u8; 32],
}

impl MerkleTree {
    fn proof_for(&self, mut index: usize) -> Vec<[u8; 32]> {
        let mut siblings = Vec::new();
        for layer in &self.layers {
            if layer.len() <= 1 {
                break;
            }
            let sibling_idx = index ^ 1;
            let sib = if sibling_idx < layer.len() {
                layer[sibling_idx]
            } else {
                // Odd node duplicated up.
                layer[index]
            };
            siblings.push(sib);
            index /= 2;
        }
        siblings
    }
}

fn build_tree(leaves: &[[u8; 32]]) -> MerkleTree {
    let mut layers: Vec<Vec<[u8; 32]>> = Vec::new();
    layers.push(leaves.to_vec());

    if leaves.len() == 1 {
        return MerkleTree {
            root: leaves[0],
            layers,
        };
    }

    loop {
        let last = layers.last().unwrap();
        if last.len() == 1 {
            break;
        }
        let mut next = Vec::with_capacity((last.len() + 1) / 2);
        let mut i = 0;
        while i < last.len() {
            if i + 1 < last.len() {
                next.push(hash_pair_sorted(&last[i], &last[i + 1]));
            } else {
                // Odd leaf: hash with itself.
                next.push(hash_pair_sorted(&last[i], &last[i]));
            }
            i += 2;
        }
        layers.push(next);
    }
    let root = *layers.last().unwrap().first().unwrap();
    MerkleTree { layers, root }
}

pub fn verify_proof(leaf: &[u8; 32], proof: &[[u8; 32]], root: &[u8; 32]) -> bool {
    let mut cur = *leaf;
    for sib in proof {
        cur = hash_pair_sorted(&cur, sib);
    }
    &cur == root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(b: u8) -> [u8; 32] {
        let mut x = [0u8; 32];
        x[0] = b;
        x
    }

    #[test]
    fn single_leaf_root_is_leaf() {
        let tree = build_tree(&[h(1)]);
        assert_eq!(tree.root, h(1));
        assert!(tree.proof_for(0).is_empty());
    }

    #[test]
    fn two_leaves() {
        let leaves = vec![h(1), h(2)];
        let tree = build_tree(&leaves);
        let proof0 = tree.proof_for(0);
        let proof1 = tree.proof_for(1);
        assert!(verify_proof(&leaves[0], &proof0, &tree.root));
        assert!(verify_proof(&leaves[1], &proof1, &tree.root));
    }

    #[test]
    fn five_leaves_each_verifies() {
        let leaves: Vec<[u8; 32]> = (1u8..=5).map(h).collect();
        let tree = build_tree(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.proof_for(i);
            assert!(verify_proof(leaf, &proof, &tree.root), "leaf {i} failed");
        }
    }

    #[test]
    fn wrong_leaf_fails() {
        let leaves = vec![h(1), h(2), h(3), h(4)];
        let tree = build_tree(&leaves);
        let proof = tree.proof_for(0);
        assert!(!verify_proof(&h(9), &proof, &tree.root));
    }

    #[test]
    fn leaf_hash_deterministic() {
        let a = hash_leaf("Alice", 100);
        let b = hash_leaf("Alice", 100);
        let c = hash_leaf("Alice", 101);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn hash_pair_sorted_is_commutative() {
        // result must be identical regardless of argument order
        let a = h(3);
        let b = h(7);
        assert_eq!(hash_pair_sorted(&a, &b), hash_pair_sorted(&b, &a));
    }

    #[test]
    fn four_leaves_all_verify() {
        let leaves: Vec<[u8; 32]> = (1u8..=4).map(h).collect();
        let tree = build_tree(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            assert!(verify_proof(leaf, &tree.proof_for(i), &tree.root), "leaf {i} failed");
        }
    }

    #[test]
    fn eight_leaves_all_verify() {
        let leaves: Vec<[u8; 32]> = (1u8..=8).map(h).collect();
        let tree = build_tree(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            assert!(verify_proof(leaf, &tree.proof_for(i), &tree.root), "leaf {i} failed");
        }
    }

    #[test]
    fn odd_seven_leaves_all_verify() {
        let leaves: Vec<[u8; 32]> = (1u8..=7).map(h).collect();
        let tree = build_tree(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            assert!(verify_proof(leaf, &tree.proof_for(i), &tree.root), "leaf {i} failed");
        }
    }

    #[test]
    fn tampered_intermediate_node_fails_verification() {
        let leaves: Vec<[u8; 32]> = (1u8..=4).map(h).collect();
        let tree = build_tree(&leaves);
        let mut proof = tree.proof_for(0);
        // flip one bit in the first sibling
        proof[0][0] ^= 0xff;
        assert!(!verify_proof(&leaves[0], &proof, &tree.root));
    }

    #[test]
    fn empty_proof_only_matches_root_when_single_leaf() {
        let leaves = vec![h(42)];
        let tree = build_tree(&leaves);
        let proof = tree.proof_for(0);
        assert!(proof.is_empty());
        assert!(verify_proof(&leaves[0], &proof, &tree.root));
        // wrong leaf against same empty proof must fail
        assert!(!verify_proof(&h(99), &proof, &tree.root));
    }

    #[test]
    fn leaf_hash_empty_wallet_zero_points() {
        // must not panic and must be deterministic
        let a = hash_leaf("", 0);
        let b = hash_leaf("", 0);
        assert_eq!(a, b);
        // different from a non-empty wallet
        assert_ne!(a, hash_leaf("x", 0));
    }

    #[test]
    fn leaf_hash_u64_max_points() {
        // must not panic
        let _ = hash_leaf("wallet", u64::MAX);
    }

    #[test]
    fn proof_length_is_logarithmic() {
        // ceil(log2(n)) for n>=2, 0 for n=1.
        fn ceil_log2(n: usize) -> usize {
            if n <= 1 { 0 } else { (usize::BITS - (n - 1).leading_zeros()) as usize }
        }
        for n in [1usize, 2, 4, 7, 8, 16, 32, 100, 1000] {
            let leaves: Vec<[u8; 32]> = (0..n).map(|i| h((i & 0xff) as u8)).collect();
            let tree = build_tree(&leaves);
            let proof_len = tree.proof_for(0).len();
            let expected = ceil_log2(n);
            assert_eq!(
                proof_len, expected,
                "n={n} expected proof length {expected}, got {proof_len}"
            );
        }
    }

    #[test]
    fn proof_index_is_position_specific() {
        // a proof for index i must NOT verify a different leaf at the same index slot
        let leaves: Vec<[u8; 32]> = (1u8..=8).map(h).collect();
        let tree = build_tree(&leaves);
        let proof_2 = tree.proof_for(2);
        let bogus_leaf = h(99);
        assert!(!verify_proof(&bogus_leaf, &proof_2, &tree.root));
    }

    #[test]
    fn swapping_two_leaves_changes_root() {
        let mut a = vec![h(1), h(2), h(3), h(4)];
        let root_a = build_tree(&a).root;
        a.swap(0, 1);
        let root_b = build_tree(&a).root;
        // Due to sorted-pair hashing, swapping siblings at the *same parent* produces the
        // same root. Verify only when the swap crosses a parent boundary.
        a.swap(0, 1); // restore
        a.swap(0, 2); // cross-parent swap
        let root_c = build_tree(&a).root;
        assert_eq!(root_a, root_b, "sibling swap should preserve root (sorted-pair)");
        assert_ne!(root_a, root_c, "cross-parent swap should change root");
    }

    #[test]
    fn adding_a_leaf_changes_root() {
        let base: Vec<[u8; 32]> = (1u8..=4).map(h).collect();
        let mut extended = base.clone();
        extended.push(h(99));
        assert_ne!(build_tree(&base).root, build_tree(&extended).root);
    }

    #[test]
    fn duplicate_leaves_still_verify() {
        let leaves = vec![h(1), h(1), h(1), h(1)];
        let tree = build_tree(&leaves);
        for i in 0..leaves.len() {
            let proof = tree.proof_for(i);
            assert!(verify_proof(&leaves[i], &proof, &tree.root));
        }
    }

    #[test]
    fn large_tree_64_leaves_all_verify() {
        let leaves: Vec<[u8; 32]> = (0u8..64).map(h).collect();
        let tree = build_tree(&leaves);
        for (i, leaf) in leaves.iter().enumerate() {
            assert!(
                verify_proof(leaf, &tree.proof_for(i), &tree.root),
                "leaf {i} failed in 64-leaf tree"
            );
        }
    }

    #[test]
    fn cross_leaf_proof_does_not_verify() {
        // proof for leaf 0 must NOT verify leaf 5 against the same root
        let leaves: Vec<[u8; 32]> = (1u8..=8).map(h).collect();
        let tree = build_tree(&leaves);
        let proof_0 = tree.proof_for(0);
        assert!(!verify_proof(&leaves[5], &proof_0, &tree.root));
    }

    #[test]
    fn truncated_proof_fails() {
        let leaves: Vec<[u8; 32]> = (1u8..=8).map(h).collect();
        let tree = build_tree(&leaves);
        let mut proof = tree.proof_for(0);
        proof.pop(); // drop the topmost sibling
        assert!(!verify_proof(&leaves[0], &proof, &tree.root));
    }

    #[test]
    fn extended_proof_fails() {
        let leaves: Vec<[u8; 32]> = (1u8..=4).map(h).collect();
        let tree = build_tree(&leaves);
        let mut proof = tree.proof_for(0);
        proof.push(h(42)); // append a junk sibling
        assert!(!verify_proof(&leaves[0], &proof, &tree.root));
    }

    #[test]
    fn determinism_same_input_same_root() {
        let leaves: Vec<[u8; 32]> = (1u8..=10).map(h).collect();
        let r1 = build_tree(&leaves).root;
        let r2 = build_tree(&leaves).root;
        assert_eq!(r1, r2);
    }

    #[test]
    fn wallet_collisions_unlikely_in_leaf() {
        // two distinct wallets at the same point count must produce different leaves
        let a = hash_leaf("WalletA", 100);
        let b = hash_leaf("WalletB", 100);
        assert_ne!(a, b);
    }
}

// Property-based tests for the Merkle tree. Each `proptest!` block generates
// hundreds of random inputs (default 256 cases per property).
#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_leaf() -> impl Strategy<Value = [u8; 32]> {
        any::<[u8; 32]>()
    }

    proptest! {
        #[test]
        fn every_leaf_in_a_tree_verifies(leaves in proptest::collection::vec(arb_leaf(), 1..=64)) {
            let tree = build_tree(&leaves);
            for (i, leaf) in leaves.iter().enumerate() {
                let proof = tree.proof_for(i);
                prop_assert!(verify_proof(leaf, &proof, &tree.root),
                    "n={} idx={} failed", leaves.len(), i);
            }
        }

        #[test]
        fn build_tree_is_deterministic(leaves in proptest::collection::vec(arb_leaf(), 1..=32)) {
            let r1 = build_tree(&leaves).root;
            let r2 = build_tree(&leaves).root;
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn hash_pair_is_commutative(a in arb_leaf(), b in arb_leaf()) {
            prop_assert_eq!(hash_pair_sorted(&a, &b), hash_pair_sorted(&b, &a));
        }

        #[test]
        fn random_leaf_substitution_fails(
            leaves in proptest::collection::vec(arb_leaf(), 2..=32),
            bogus in arb_leaf(),
            idx in any::<usize>(),
        ) {
            let tree = build_tree(&leaves);
            let i = idx % leaves.len();
            prop_assume!(bogus != leaves[i]);
            let proof = tree.proof_for(i);
            prop_assert!(!verify_proof(&bogus, &proof, &tree.root));
        }

        #[test]
        fn appending_a_leaf_changes_the_root(
            leaves in proptest::collection::vec(arb_leaf(), 1..=16),
            extra in arb_leaf(),
        ) {
            let r1 = build_tree(&leaves).root;
            let mut extended = leaves.clone();
            extended.push(extra);
            let r2 = build_tree(&extended).root;
            // It's astronomically unlikely (2^-256) that adding a different leaf preserves
            // the root, so this should always hold for random inputs.
            prop_assert_ne!(r1, r2);
        }

        #[test]
        fn leaf_hash_collisions_require_input_collisions(
            w1 in "[A-Za-z0-9]{1,32}",
            w2 in "[A-Za-z0-9]{1,32}",
            p1 in any::<u64>(),
            p2 in any::<u64>(),
        ) {
            // hash_leaf is injective for distinct (wallet, points) pairs (under SHA-256).
            prop_assume!(w1 != w2 || p1 != p2);
            prop_assert_ne!(hash_leaf(&w1, p1), hash_leaf(&w2, p2));
        }
    }
}

// Formal verification harnesses (run with `cargo kani`). Cargo skips this module
// in normal builds; Kani sets the `kani` cfg automatically when verifying.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // The core safety property of the Merkle scheme: a freshly generated proof
    // for any index in a small tree verifies against that tree's root, bit-exact.
    // Bounded to 4 leaves so Kani's symbolic execution terminates in reasonable time.
    #[kani::proof]
    #[kani::unwind(8)]
    fn merkle_proof_verifies_for_any_4_leaf_tree() {
        let l0: [u8; 32] = kani::any();
        let l1: [u8; 32] = kani::any();
        let l2: [u8; 32] = kani::any();
        let l3: [u8; 32] = kani::any();
        let leaves = vec![l0, l1, l2, l3];
        let tree = build_tree(&leaves);

        let idx: usize = kani::any();
        kani::assume(idx < 4);
        let proof = tree.proof_for(idx);
        assert!(verify_proof(&leaves[idx], &proof, &tree.root));
    }

    // The sorted-pair hash must be commutative: hash(a,b) == hash(b,a) for any a, b.
    // Without this property our proofs would need to track left/right ordering, which
    // would break the simple sibling-list encoding the Solana program expects.
    #[kani::proof]
    fn hash_pair_sorted_is_commutative_kani() {
        let a: [u8; 32] = kani::any();
        let b: [u8; 32] = kani::any();
        assert_eq!(hash_pair_sorted(&a, &b), hash_pair_sorted(&b, &a));
    }

    // hash_leaf is deterministic — same wallet, same points → same hash.
    #[kani::proof]
    fn hash_leaf_is_deterministic_kani() {
        let pts: u64 = kani::any();
        let w = "kani-test-wallet";
        assert_eq!(hash_leaf(w, pts), hash_leaf(w, pts));
    }

    // hash_leaf is injective on the points field: different points → different hash
    // (under SHA-256, with negligible collision probability — Kani proves the
    // function itself doesn't accidentally collapse them).
    #[kani::proof]
    fn hash_leaf_distinct_points_distinct_hashes() {
        let p1: u64 = kani::any();
        let p2: u64 = kani::any();
        kani::assume(p1 != p2);
        let h1 = hash_leaf("w", p1);
        let h2 = hash_leaf("w", p2);
        assert!(h1 != h2);
    }

    // Tampering ANY byte of the leaf must break verification against the same
    // proof + root. Bounded to a 2-leaf tree to keep Kani tractable.
    #[kani::proof]
    #[kani::unwind(4)]
    fn tampered_leaf_byte_fails_verification() {
        let l0: [u8; 32] = kani::any();
        let l1: [u8; 32] = kani::any();
        let leaves = vec![l0, l1];
        let tree = build_tree(&leaves);
        let proof = tree.proof_for(0);

        // Pick any byte position to flip.
        let idx: usize = kani::any();
        kani::assume(idx < 32);
        let bit: u8 = kani::any();
        kani::assume(bit != 0);

        let mut tampered = l0;
        tampered[idx] ^= bit;
        kani::assume(tampered != l0);
        assert!(!verify_proof(&tampered, &proof, &tree.root));
    }

    // A truncated proof never verifies (would-be sibling missing).
    #[kani::proof]
    #[kani::unwind(8)]
    fn truncated_proof_fails() {
        let l0: [u8; 32] = kani::any();
        let l1: [u8; 32] = kani::any();
        let l2: [u8; 32] = kani::any();
        let l3: [u8; 32] = kani::any();
        let leaves = vec![l0, l1, l2, l3];
        let tree = build_tree(&leaves);

        let mut proof = tree.proof_for(0);
        // Original proof is non-empty for 4 leaves. Drop the top sibling.
        let _ = proof.pop();
        assert!(!verify_proof(&l0, &proof, &tree.root));
    }

    // Building the same tree twice gives the same root — vital for snapshot
    // determinism across server restarts.
    #[kani::proof]
    #[kani::unwind(4)]
    fn build_tree_is_deterministic() {
        let l0: [u8; 32] = kani::any();
        let l1: [u8; 32] = kani::any();
        let l2: [u8; 32] = kani::any();
        let leaves = vec![l0, l1, l2];
        let t1 = build_tree(&leaves);
        let t2 = build_tree(&leaves);
        assert!(t1.root == t2.root);
    }

    // hash_pair_sorted only depends on the {a, b} set, not on order or content
    // beyond that. Concretely: hash(a, b) == hash(b, a) ALWAYS.
    #[kani::proof]
    fn hash_pair_sorted_order_independent() {
        let a: [u8; 32] = kani::any();
        let b: [u8; 32] = kani::any();
        assert_eq!(hash_pair_sorted(&a, &b), hash_pair_sorted(&b, &a));
    }
}
