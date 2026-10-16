use super::*;

#[test]
fn sha256_matches_the_standard_vectors_whole_and_in_pieces() {
    // FIPS 180-2 test vectors.
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    let mut h = Sha256::default();
    h.update(b"a");
    h.update(b"bc");
    assert_eq!(h.finish_hex(), sha256_hex(b"abc"));
}
