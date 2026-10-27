# Read-only Clippy E0275 diagnosis

The observed failure is specific to the `clippy::useless_conversion` probe in this run. The default all-target log fails while checking `crystal-bevy` lib tests; the same library/tests check with only the additional `-A clippy::useless_conversion` completes. No source edits, Cargo invocations, dependency changes or recursion-limit changes were made for this diagnosis.

## Actual local evidence

- [Default failure](../../../.local/current510-native-pp-final-bevy-clippy.log), lines 9686–9697: E0275 evaluating `&itertools::groupbylazy::ChunkBy<_,_,_>: IntoIterator`, with 125 recursive `&Res<...>` requirements. No expression span is attached. The library check had already completed; the failure is the lib-test target.
- [Root's isolated check](../../../.local/current510-native-pp-clippy-no-useless-conversion.log), lines 11433–11434: library/tests complete in 59.67 seconds, 509 existing lib-test warnings, with the one additional lint allowance.
- Installed pinned compiler: `rustc 1.99.0 (b940084d7 2026-09-28)`, full commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, aarch64-apple-darwin. This was read using the existing executable's `-Vv`, without building.
- The saved `.long-type` file shows a trait-search chain of `Res` wrappers ending at `ChunkBy<_,_,_>`. It does not identify an authored expression or prove that game state constructs such a value.

## Source-level mechanism

The [exact pinned Clippy source](https://github.com/rust-lang/rust/blob/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/src/tools/clippy/clippy_lints/src/useless_conversion.rs#L79-L115) implements `into_iter_bound`: when considering removal of `.into_iter()` from a generic argument, it substitutes the original receiver into the parent function's other trait bounds and asks `predicate_must_hold_modulo_regions`. The obligation uses `ObligationCause::dummy()`. This is a concrete possible source of a spanless trait-resolution failure; no diagnostic backtrace proves this exact function was the failing invocation in this run.

The local dependency `/Users/ryanculligan/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.14.2/src/change_detection.rs:536` implements `IntoIterator for &Res<T>` conditional on `&T: IntoIterator`. An unconstrained trait query can consequently explore successive `Res` wrappers. `itertools`' borrowed `ChunkBy` is another candidate for `IntoIterator`. Their appearance in the diagnostic is consistent with solver exploration, rather than gameplay recursion or map geometry.

## Recent code inspected

The R12 test leaf's sole explicit `.into_iter()` is `neighbors(from).into_iter().find_map(...)` at `crates/crystal-bevy/src/bevy_shell/tests/r12_shelter.rs:172`. `neighbors` returns the concrete array `[(Point, GameButton); 4]` (`authored_route_fixture.rs:239`), and this call is the receiver of `find_map`, not a generic argument. It does not fit the risky `into_iter_bound` route described above. Its two `.into()` calls have concrete `String` destinations. The added PP diagnostics' iterator chain uses `.iter().rev().take(...).collect::<Vec<_>>()`, without `.into_iter()`.

The recent PP target helpers in `item_target_capabilities.rs` use scalar option/field checks, with no iterator conversions or generic trait bounds. The default library target succeeding also weighs against claiming that helper is the direct trigger. I found no concrete new expression that can honestly be named as the failing source location.

## Recommendation

Retain the narrowly scoped command-line `-A clippy::useless_conversion` for this pinned-toolchain verification, and report the default Clippy failure alongside the isolated passing check and passing production tests. Do not increase recursion depth, disable other lints, add a crate-wide allowance, remove gameplay assertions, or rewrite unrelated iterators without a reproducer. A future diagnostic build or a minimized source bisect could locate the exact expression; neither was run here.

## Frozen map evidence

R13 v02 remains unchanged by this diagnosis. Its `source-proof-3457550d3dc72bf615f58b05639b87164116956691f5e77cd0a7099720e33d68.json` records 28/28 checks. All nine `negative-proof/<case>/MnR13Cave1F.candidate.json` variants retain their source-check report, whole originals and seven 160px / 4× camera pairs. No broken source replaces the accepted candidate or live route.
