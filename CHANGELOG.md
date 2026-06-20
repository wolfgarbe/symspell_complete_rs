# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.4] - 2026-06-20

`gx`-feature flag renamed to `gxhash`.

### Changed

## [0.0.3] - 2026-06-09

### Changed

- Explicit activation of new `gx`-feature flag for high-performance hashing via `gxhash`, both for `x86_64` and `aarch64`. Enable the `gx`-feature:
  1. If compiling for x86_64 AND the user explicitly targeted AES/SSE2: `#[cfg(all(target_arch = "x86_64", target_feature = "aes", target_feature = "sse2"))]` or
  2. If compiling for ARM64 AND the user explicitly targeted AES/NEON`#[cfg(all(target_arch = "aarch64", target_feature = "aes", target_feature= "neon"))]`
  3. Otherwise fallback to `ahash`.
  4. In addition to enabling the `gx`-feature flag you need a .cargo/config.toml 
    [build]
    rustflags = ["-C", "target-cpu=native"]
    rustdocflags = ["-C", "target-cpu=native"]
    or
    [build]
    rustflags = ["-C", "target-feature=+aes,+sse2"]
    rustdocflags = ["-C", "target-cpu=+aes,+sse2"]

## [0.0.2] - 2025-12-05

### Changed

- get_dictionary_size: Number of words in the dictionary with a frequency count >= count_threshold (previously including words with a frequency count < count_threshold).
- Numbers are ignored. They are not added to the dictionary, and the are always deemed correct. But they are still used for multi-term completion.

### Added

- get_candidates_size: Number of words in the words candidate list, including those with a frequency count < count_threshold.

## [0.0.1] - 2025-12-05

### Added

- Initial release
