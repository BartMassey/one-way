use rand::*;

pub fn random(r: u64) -> u64 {
    let mut prng = thread_rng();
    prng.gen_range(0, r)
}
