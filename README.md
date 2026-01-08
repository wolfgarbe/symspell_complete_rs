SymSpellComplete<br>
[![Crates.io](https://img.shields.io/crates/v/symspell_complete_rs.svg)](https://crates.io/crates/symspell_complete_rs)
[![Downloads](https://img.shields.io/crates/d/symspell_complete_rs.svg?style=flat-square)](https://crates.io/crates/symspell_complete_rs)
[![MIT License](https://img.shields.io/github/license/wolfgarbe/symspell_complete_rs.svg)](https://github.com/wolfgarbe/symspell_complete_rs/blob/main/LICENSE)
[![Documentation](https://docs.rs/symspell_complete_rs/badge.svg)](https://docs.rs/symspell_complete_rs)
========

**SymSpellComplete**, a typo-tolerant autocomplete library in Rust.

⚠️ Work-in-progress. Currently no third party use recommended, as the interface is not yet stable, and not all advertised features are implemented yet.  
Currently only intended to be used by [SeekStorm search library](https://github.com/SeekStorm/SeekStorm) v1.2.0. 


Like a combination of [SymSpell](https://github.com/wolfgarbe/SymSpell) and [PruningRadixTrie](https://github.com/wolfgarbe/PruningRadixTrie), but better!

- Query terms might contain spelling errors, even in the first letter of an incomplete term: `blu kura` ➔ `blue curacao`
- Handles missing and mistakenly inserted spaces: `modernart` ➔ `modern art`
- Supports differing order between query and completion terms: `university dre` ➔ `dresden university of technology`
- Allows out-of-vocabulary terms (not all query terms are present in completions): `teal merino cardi` ➔ `merino cardigan`

When integrated into the [**SeekStorm**](https://github.com/SeekStorm/SeekStorm) full-text search library & multi-tenancy server, the **completions** will be derived in real-time **from indexed documents**, not from a **query log**:

- works, even if no query log is available, especially for domain-specific, newly created indices or few users.
- works for new or domain-specific terms.
- allows out-of-the-box domain specific suggestions.
- prevents inconsistencies between completions and index content.
- suggestions tailored per index.
- Works for the long tail of queries that never reached a log.
- possible drawback: content-driven vs. usage-driven suggestion ranking.
- Ghosting: highlighting the suggested text within the search box in the UI.
 


If you like SymSpellComplete, try [**SeekStorm**](https://github.com/SeekStorm/SeekStorm) - a sub-millisecond full-text search library & multi-tenancy server in Rust (Open Source).

<br>

```text
Copyright (c) 2025 Wolf Garbe
Version: 0.0.1
Author: Wolf Garbe <wolf.garbe@seekstorm.com>
Maintainer: Wolf Garbe <wolf.garbe@seekstorm.com>
URL: https://github.com/wolfgarbe/symspell_complete_rs
Description: https://seekstorm.com/blog/query-auto-completion-(QAC)/

MIT License

Copyright (c) 2025 Wolf Garbe

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated 
documentation files (the "Software"), to deal in the Software without restriction, including without limitation 
the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, 
and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

https://opensource.org/licenses/MIT
```

---
