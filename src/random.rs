use rand::*;

pub fn random(r: u64) -> u64 {
    let mut prng = rng();
    prng.random_range(0..r)
}
