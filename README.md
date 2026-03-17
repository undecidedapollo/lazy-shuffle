# lazy-shuffle

Lazy Fisher-Yates shuffle iterator - yielding random permutations without allocating the full sequence

## Usage

### (std) Default OS RNG

```rust
use lazy_shuffle::ShuffleExt;

let items = ["a", "b", "c", "d", "e"];

// each element exactly once, in random order
for val in items.shuffled() {
    println!("{val}"); 
}
```

### (no-std) Seeded or custom RNG 

Use a fixed seed for a deterministic shuffle:

```rust
use lazy_shuffle::ShuffleExt;

let items = ["a", "b", "c", "d", "e"];

for val in items.shuffled_with_seed(42) {
    let _ = val;
}

```

Or use a specific rng algorithm:

```rust
use lazy_shuffle::ShuffleExt;
use rand::SeedableRng;
use rand::rngs::SmallRng;

let items = ["a", "b", "c", "d", "e"];
let rng = SmallRng::seed_from_u64(123);

for val in items.shuffled_with_rng(rng) {
    let _ = val;
}
```
