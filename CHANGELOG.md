# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2] - 2025-12-05

### Changed

- get_dictionary_size: Number of words in the dictionary with a frequency count >= count_threshold (previously including words with a frequency count < count_threshold).
- Numbers are ignored. They are not added to the dictionary, and the are always deemed correct. But they are still used for multi-term completion.

### Added

- get_candidates_size: Number of words in the words candidate list, including those with a frequency count < count_threshold.

## [0.0.1] - 2025-12-05

### Added

- Initial release
