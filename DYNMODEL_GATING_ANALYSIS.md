# DynModel gating — structural analysis

Context: [ublox-rs/ublox#260](https://github.com/ublox-rs/ublox/issues/260)
([PR #230 comment](https://github.com/ublox-rs/ublox/pull/230#discussion_r2626052386))

## The surface problem

`cfg_nav5.rs` gates `DynModel::Bike/Mower/Escooter` behind
`#[cfg(feature = "ubx_proto31")]`. That is factually wrong: those values
appeared on F9P (proto 27.31), are documented on F9T (29.25), F9K (30.50)
and F9R (33.40), and are *absent* from the only proto-31 device — the
ZED-F9H, whose 2020 ICD has not been touched since. The literal fix is to
retag them to `ubx_proto27`.

## The real problem

That fix is a coat of paint over a structural mistake. The dynModel matrix
isn't a function of protocol version alone. It depends on **at least three
orthogonal axes**:

1. **Protocol version** — roughly monotonic, but not totally ordered.
   F9H sits at 31.12 (2020) and supports *less* than F9P at 27.50 (2024).
2. **Device family** — M8 / F9P / F9T / F9K / F9H / M9 / F9R / M10 / F10 /
   F9L / X20. Each family carries its own enum-value subset.
3. **Firmware revision within family** — F9P-27.12 lacks BIKE; F9P-27.31+
   has it. Same proto major, different capability.

And a fourth, slightly different, axis:

4. **Configuration method** — legacy `UBX-CFG-NAV5` message vs.
   `CFG-NAVSPG-DYNMODEL` config-database key. Some devices accept one, some
   both. Independent of which enum values they accept.

A Cargo feature is a one-dimensional, build-time switch. The matrix it is
being asked to encode is three- or four-dimensional and per-device. The
mechanism cannot fit the problem. Every "fix" within the current scheme
will be wrong again the next firmware drop.

## Who actually needs what?

Three user populations are conflated by the current design:

- **A — single-device firmware author.** Wants a compile-time error if
  they ask their fixed F9P to do something it can't. Today's feature gates
  *try* to serve them but pick the wrong axis (proto, not device + firmware).
- **B — general application / driver author.** Talks to whatever the user
  plugs in. Capability must be discovered at runtime. Compile-time gating
  is actively harmful: it forces them to pick one proto and excludes the rest.
- **C — packet codec / log replay user.** Decodes captures from any
  receiver. Needs every variant present, always. Gating breaks them.

The crate is implicitly optimised for A but on the wrong axis, paying the
cost for B and C.

## The candid framing

The most honest read from the issue thread: *"It's not clear what value is
achieved from the protocol gating currently."* `pyubx2` doesn't gate at all
and is fine. The gating pretends to enforce a constraint the receiver will
already enforce via NAK. The crate is paying a large maintenance cost for
an illusion of safety.

## Novel reframings

Ordered roughly from least to most disruptive.

### 1. Stop gating types; gate behaviour

Make `DynModel` always contain every variant. Move the validity check from
the type system to the device handle:

```rust
device.set_dyn_model(DynModel::Bike)
   .map_err(|NotSupported { variant, since_proto, device_proto, .. }| …)
```

The receiver NAKs anyway. Catch the NAK, attach a useful context struct,
done. This removes the entire feature-gate mess in one step and is honest
about where the constraint actually lives.

### 2. Split the two axes that are currently conflated

"Which message/key do I use to write this?" and "Which enum values will
this device accept?" are independent questions. Today they're both bundled
under one proto feature. Splitting them collapses most of the table:

- A `ConfigDialect` trait/enum picks `CFG-NAV5` vs `CFG-VALSET` per device.
- A `DynModel` capability check validates the enum value at write time.

This is a refactor that doesn't yet require runtime probing.

### 3. Runtime probe + typestate — compile-time guarantees *after* discovery

The compile-time gating that population A wants is achievable, but the
discriminator has to be the **device**, not the **build**:

```rust
let raw  = UbloxTransport::new(port);
let dev  = raw.probe().await?;                  // reads MON-VER
let f9p  = dev.expect_family::<F9P>()?;         // ZST family marker
let f9p_modern = f9p.expect_firmware_ge("1.32")?;
f9p_modern.set_dyn_model(DynModel::Bike);       // compile-error on F9H-typed handle
```

Family and firmware are zero-sized type parameters; capability is encoded
via sealed marker traits. Single-device users get *real* compile-time
safety; multi-device users skip the `expect_*` step and validate at
runtime. The codec layer (population C) never sees any of this.

### 4. Generate the capability matrix from the spec, don't hand-curate it

The dynModel table in the issue thread was hand-derived from ~16 ICD PDFs.
It will rot the moment u-blox ships SPG-5.40. Replace the scattered
`#[cfg]` attributes with one owned artifact:

- a YAML/TOML "capability database" —
  `(device, proto, firmware) → { supported_dyn_models, supported_cfg_keys, config_dialect }`,
- consumed by `build.rs` or a proc-macro that emits the enums, the
  validation tables, and the marker-trait `impl`s.

Each ICD update becomes a one-file PR with a citation, not surgery across
the codebase. Optional bonus: the YAML is independently useful —
publishable as its own crate or shared JSON dataset for `pyubx2` /
`ubxlib` / etc.

### 5. Stop owning the matrix entirely

Take the `pyubx2` stance: the crate is a Rust codec for u-blox UBX, not a
device-compatibility oracle. Parse and emit everything, validate nothing
about device support, surface receiver NAKs with structured context. Ship
a separate optional `ublox-compat` crate for users who want the lookup.
Lowest cost, most honest about scope, accepts that the matrix will always
lag.

## Recommendation

Combine (1) + (2) + (4) as the durable path; do (3) as a follow-on for
users who want it.

- **Now (this issue):** retag BIKE/MOWER/ESCOOTER to `ubx_proto27` as a
  stopgap, but file a tracking issue and stop adding new gates of this
  shape.
- **Next:** remove feature-gated *variants*. Replace with a capability
  table consulted at write time, returning structured `NotSupported`
  errors. Split config-dialect from value-validity.
- **Later:** move the capability table to a generated artifact from a YAML
  truth file. Optionally layer a typestate `Device<Family, Firmware>` API
  on top for the single-device-firmware audience.

## The core insight

**Cargo features were chosen as a runtime-capability mechanism. They
can't be one. The discriminator (device + firmware) isn't known at build
time, so the discrimination has to live at the layer where it is — the
connected device.**
