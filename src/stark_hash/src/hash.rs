use stark_curve::{AffinePoint, FieldElement, ProjectivePoint, PEDERSEN_P0};

use bitvec::{field::BitField, slice::BitSlice};

use crate::Felt;

include!(concat!(env!("OUT_DIR"), "/curve_consts.rs"));

/// Computes the [Starknet Pedersen hash] on `a` and `b` using precomputed points.
///
/// [Starknet Pedersen hash]: https://docs.starkware.co/starkex-v3/crypto/pedersen-hash-function
pub fn stark_hash(a: Felt, b: Felt) -> Felt {
    let a = FieldElement::from(a).into_bits();
    let b = FieldElement::from(b).into_bits();

    // Preprocessed material is lookup-tables for each chunk of bits
    let table_size = (1 << CURVE_CONSTS_BITS) - 1;
    let add_points = |acc: &mut ProjectivePoint, bits: &BitSlice<_, u64>, prep: &[AffinePoint]| {
        bits.chunks(CURVE_CONSTS_BITS)
            .enumerate()
            .for_each(|(i, v)| {
                let offset: usize = v.load_le();
                if offset > 0 {
                    // Table lookup at 'offset-1' in table for chunk 'i'
                    acc.add_affine(&prep[i * table_size + offset - 1]);
                }
            });
    };

    // Compute hash
    let mut acc = PEDERSEN_P0;
    add_points(&mut acc, &a[..248], &CURVE_CONSTS_P1); // Add a_low * P1
    add_points(&mut acc, &a[248..252], &CURVE_CONSTS_P2); // Add a_high * P2
    add_points(&mut acc, &b[..248], &CURVE_CONSTS_P3); // Add b_low * P3
    add_points(&mut acc, &b[248..252], &CURVE_CONSTS_P4); // Add b_high * P4

    // Convert to affine
    let result = AffinePoint::from(&acc);

    // Return x-coordinate
    Felt::from(result.x)
}

