# tade clean-checkout gate

date: 2026-09-24
revision: b1f1c2ca70cb4d65d32272f8ee15cf0657c9d485 (origin/main)
property: launch.md row 39 — every phase-1 component builds and tests from a
clean checkout of its default branch, no dead path, no version pin behind a
sibling, no crate present only in an owner's working tree.

tade (`impl/rust`) was not on the 2026-09-22/23 sweep's list of repos
failing this gate (bbg, cybergraph, foculus, mudra, neuron, nox, prysm,
zheng, true-cyber, glia, honeycrisp, rune, vault, radio), and it has no
prior row-39 launch PR, so this measures it directly, the method hemera's
row-39 audit and tru#24 used for the same row.

tade's crate has a single external dependency (`bytes`) and no path
dependency on a sibling repository — the crate the tape→tade rename made
every dependent (bbg, cybergraph, prysm, radio, ...) pin by version, not by
path, so tade itself has nothing to resolve against a sibling checkout.

worktree: `git worktree add ... origin/main` (no local uncommitted state;
the owner's local checkout carries 1 unpushed commit ahead of this
revision and was not used).

```
$ RUSTC_BOOTSTRAP=1 cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s

$ RUSTC_BOOTSTRAP=1 cargo test
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

no warnings, no errors. result: tade builds and tests clean from
`origin/main`. this closes tade's own slice of row 39; the row stays open
until every repo the sweep found broken has its fix merged, and every
tade dependent resolving it correctly by version is untested here.
