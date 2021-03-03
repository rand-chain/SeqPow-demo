#[macro_use]
extern crate criterion;

use criterion::Criterion;
use rug::Integer;
use seq_pow::{pie19, util};

// FOR BENCHMARKING ONLY
// NO SECURITY CHECK
pub fn verify(
    modulus: &Integer,
    g: &Integer,
    y: &Integer,
    total_num_steps: u64,
    pi_list: &Vec<Integer>,
    pubkey: &ecvrf::VrfPk,
    target: &Integer,
) -> bool {
    let hstate = util::h_state(modulus, pubkey, y);
    // if !util::validate_difficulty(&hstate, target) {
    //     return false;
    // }
    util::validate_difficulty(&hstate, target);

    let (mut x_i, mut y_i) = (g.clone(), y.clone());
    let mut t = total_num_steps;
    let two: Integer = 2u64.into();
    for mu_i in pi_list {
        let r_i = util::hash_fs(modulus, &[&x_i, &y_i, &mu_i]);

        let xi_ri = x_i.clone().pow_mod(&r_i, modulus).unwrap();
        x_i = (xi_ri * mu_i.clone()).div_rem_floor(modulus.clone()).1;

        let mui_ri = mu_i.clone().pow_mod(&r_i, modulus).unwrap();
        y_i = (mui_ri * y_i.clone()).div_rem_floor(modulus.clone()).1;

        t = t / 2;
        if (t % 2 != 0) && (t != 1) {
            t += 1;
            y_i = y_i.clone().pow_mod(&two, modulus).unwrap();
        }
    }

    y_i == x_i.pow_mod(&two, modulus).unwrap()
}

fn bench_pie19(c: &mut Criterion) {
    let bench_solve = |c: &mut Criterion,
                       num_steps: u64,
                       modulus: &Integer,
                       state: &Integer,
                       pubkey: &ecvrf::VrfPk,
                       target: &Integer| {
        c.bench_function(
            &format!("pie19::solve() with num_steps {}", num_steps),
            move |b| b.iter(|| pie19::solve(modulus, state, num_steps, pubkey, target)),
        );
    };
    let bench_prove =
        |c: &mut Criterion, num_steps: u64, modulus: &Integer, g: &Integer, y: &Integer| {
            c.bench_function(
                &format!("pie19::prove() with num_steps {}", num_steps),
                move |b| b.iter(|| pie19::prove(modulus, g, y, num_steps)),
            );
        };
    let bench_verify = |c: &mut Criterion,
                        num_steps: u64,
                        modulus: &Integer,
                        g: &Integer,
                        y: &Integer,
                        pi_list: &Vec<Integer>,
                        pubkey: &ecvrf::VrfPk,
                        target: &Integer| {
        c.bench_function(
            &format!("pie19::verify() with num_steps {}", num_steps),
            move |b| b.iter(|| verify(modulus, g, y, num_steps, pi_list, pubkey, target)),
        );
    };

    // RSA-2048 modulus, taken from [Wikipedia](https://en.wikipedia.org/wiki/RSA_numbers#RSA-2048).
    const MODULUS: &str =
      "251959084756578934940271832400483985714292821262040320277771378360436620207075955562640185258807\
      8440691829064124951508218929855914917618450280848912007284499268739280728777673597141834727026189\
      6375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172\
      6546322822168699875491824224336372590851418654620435767984233871847744479207399342365848238242811\
      9816381501067481045166037730605620161967625613384414360383390441495263443219011465754445417842402\
      0924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951\
      378636564391212010397122822120720357";
    let modulus = Integer::from_str_radix(MODULUS, 10).unwrap();

    // use 256-bit for block header hash
    const PREV_BLOCK_HASH: &str =
        "1eeb30c7163271850b6d018e8282093ac6755a771da6267edf6c9b4fce9242ba";
    const TARGET_HASH: &str = "07fb30c7163271850b6d018e8282093ac6755a771da6267edf6c9b4fce9242ba";

    let seed_hash = Integer::from_str_radix(PREV_BLOCK_HASH, 16).unwrap();
    let seed = Integer::from(seed_hash.div_rem_floor(modulus.clone()).1);
    println!("seed:\t\t0x{:064x}", seed);

    let target_hash = Integer::from_str_radix(TARGET_HASH, 16).unwrap();
    let target = Integer::from(target_hash.div_rem_floor(modulus.clone()).1);
    println!("target:\t\t0x{:064x}", target);
    println!("");

    let (_, pubkey) = ecvrf::keygen();
    let g = util::h_g(&modulus, &pubkey, &seed);

    let num_steps_arr = [
        1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 64_000, 128_000, 256_000,
    ];

    for &num_steps in &num_steps_arr {
        let (y, _) = pie19::solve(&modulus, &g, num_steps, &pubkey, &target);
        let pi_list = pie19::prove(&modulus, &g, &y, num_steps);

        bench_solve(c, num_steps, &modulus, &g, &pubkey, &target);
        bench_prove(c, num_steps, &modulus, &g, &y);
        bench_verify(c, num_steps, &modulus, &g, &y, &pi_list, &pubkey, &target);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_pie19
}
criterion_main!(benches);
