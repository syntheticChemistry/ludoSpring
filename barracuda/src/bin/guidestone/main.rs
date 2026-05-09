// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! ludoSpring guideStone — legacy entry point.
//!
//! This binary delegates to [`ludospring_barracuda::certification::certify`].
//! Prefer using `ludospring certify` (the eukaryotic UniBin) instead.
//!
//! Retained for backwards compatibility with CI scripts and plasmidBin.

fn main() {
    let result = ludospring_barracuda::certification::certify(3);
    std::process::exit(result.exit_code());
}
