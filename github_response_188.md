I've been thinking about this over the last day or so.  The manual approach works well but would be tedious to go back and implement for all existing packet types.

I've looked into the feasibility of extending `ublox_derive` to auto-generate fuzz strategies. The `#[ubx_packet_recv]` macro already parses all the field definitions, so it knows field names, types, sizes, class/id, and importantly the `map_type` mappings to enums. That should be everything you need to generate proptest strategies that build valid UBX frames.

The approach would be to add a new output module to `ublox_derive` that generates `fuzz_*_strategy()` methods for each packet type, gated behind `#[cfg(any(test, feature = "fuzz"))]`.

So conceptually, given a packet definition like this:

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x09, fixed_payload_len = 60)]
struct MonHw {
    pin_sel: u32,
    pin_bank: u32,
    #[ubx(map_type = AntennaStatus)]  // already exists - could inform valid values
    a_status: u8,
    // ...
}
```

you could automatically get something like this:

```rust
impl MonHw {
    pub fn fuzz_frame_strategy() -> impl Strategy<Value = Vec<u8>> { ... }
    pub fn fuzz_frame_chaos_strategy() -> impl Strategy<Value = Vec<u8>> { ... }
}
```

This would work for all existing `recv` packet types automatically, and future packet types would get fuzz coverage with no additional effort.

Some open questions before going too far down this path:

1. **Semantic validity vs chaos testing** — Do we want strategies that produce only semantically valid data (eg months 1-12, valid enum values) which I think is the focus of the current approach, or do you also want "chaos" strategies that test parser robustness with arbitrary bytes? Probably both?

2. **How much validity info to encode** — The existing `map_type` attributes already provide enum constraints. Adding explicit range hints (`#[ubx(valid_range = 1..=12)]`) is possible but adds annotation burden. Is it worth it, or is enum-awareness + chaos testing sufficient?

There are more considerations to resolve, but curious what the thoughts are about the general direction before going deeper into the problem.