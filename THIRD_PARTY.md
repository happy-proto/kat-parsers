# Third-Party Sources

This repository contains source-only grammar snapshots externalized from `kat`, plus the generated parser artifacts committed inside per-language crates for direct git consumption by `kat`. `crates/tree-sitter-kat-parsers/` is now only an aggregate re-export crate, while shared build support lives under `crates/kat-parser-common/`.

## Shared Tree-sitter C headers

- Shared support crate: `crates/kat-parser-common`
- Shared headers:
  - `crates/kat-parser-common/include/tree_sitter/parser.h`
  - `crates/kat-parser-common/include/tree_sitter/alloc.h`
  - `crates/kat-parser-common/include/tree_sitter/array.h`
- Provenance:
  - `parser.h` matches `tree-sitter 0.26.8`'s `src/parser.h`
  - `alloc.h` and `array.h` match the vendored copies from `svkozak/tree-sitter-coffeescript` at `3bb4dbd68ca926c76b3baadb529da4de3726ea37`
- Notes: these headers are shared build-time support assets extracted out of the old monolithic bundle so each language crate can compile independently without implicit cross-language include paths.

## CoffeeScript

- Upstream repository: `https://github.com/svkozak/tree-sitter-coffeescript`
- Upstream revision: `3bb4dbd68ca926c76b3baadb529da4de3726ea37`
- License: `MIT`
- Imported source assets:
  - `grammar.js`
  - `src/scanner.c`
  - `src/tree_sitter/alloc.h`
  - `src/tree_sitter/array.h`
  - `src/tree_sitter/parser.h`
  - `LICENSE.txt`
- Local generated artifacts:
  - `crates/tree-sitter-kat-coffeescript/vendor/coffeescript/.parser-inputs`
  - `crates/tree-sitter-kat-coffeescript/vendor/coffeescript/parser.c`
  - `crates/tree-sitter-kat-coffeescript/vendor/coffeescript/scanner.c`
  - `crates/tree-sitter-kat-coffeescript/vendor/coffeescript/grammar.json`
  - `crates/tree-sitter-kat-coffeescript/vendor/coffeescript/node-types.json`

## Crystal

- Upstream repository: `https://github.com/crystal-lang-tools/tree-sitter-crystal`
- Upstream revision: `50ca9e6fcfb16a2cbcad59203cfd8ad650e25c49`
- License: `MIT`
- Imported source assets:
  - `grammar.js`
  - `src/scanner.c`
  - `src/unicode.c`
  - `src/tree_sitter/alloc.h`
  - `src/tree_sitter/array.h`
  - `src/tree_sitter/parser.h`
  - `LICENSE`
- Local generated artifacts:
  - `crates/tree-sitter-kat-crystal/vendor/crystal/.parser-inputs`
  - `crates/tree-sitter-kat-crystal/vendor/crystal/parser.c`
  - `crates/tree-sitter-kat-crystal/vendor/crystal/scanner.c`
  - `crates/tree-sitter-kat-crystal/vendor/crystal/unicode.c`
  - `crates/tree-sitter-kat-crystal/vendor/crystal/grammar.json`
  - `crates/tree-sitter-kat-crystal/vendor/crystal/node-types.json`

## actionscript

Imported source assets:
- `upstreams/actionscript/grammar.js`

Original provenance carried over from `kat`:
Source: [jcs090218/tree-sitter-actionscript](https://github.com/jcs090218/tree-sitter-actionscript)
  Revision: `12fc0c4c822c6edd924c13b328a93fe69454b299`
  License: MIT
  Notes: repository-local copy of the upstream ActionScript grammar and highlights query, kept as a dedicated runtime for `.as` source files.

Local generated artifacts:
- `crates/tree-sitter-kat-actionscript/vendor/actionscript/.parser-inputs`
- `crates/tree-sitter-kat-actionscript/vendor/actionscript/grammar.json`
- `crates/tree-sitter-kat-actionscript/vendor/actionscript/node-types.json`
- `crates/tree-sitter-kat-actionscript/vendor/actionscript/parser.c`

## apache

Imported source assets:
- `upstreams/apache/grammar.js`

Original provenance carried over from `kat`:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for Apache-style configuration files.

Local generated artifacts:
- `crates/tree-sitter-kat-apache/vendor/apache/.parser-inputs`
- `crates/tree-sitter-kat-apache/vendor/apache/grammar.json`
- `crates/tree-sitter-kat-apache/vendor/apache/node-types.json`
- `crates/tree-sitter-kat-apache/vendor/apache/parser.c`

## applescript

Imported source assets:
- `upstreams/applescript/grammar.js`
- `upstreams/applescript/scanner.c`

Original provenance carried over from `kat`:
Source: [waddie/tree-sitter-applescript](https://github.com/waddie/tree-sitter-applescript)
  Revision: `adff3f4de87033350050232c8dd23947c7b34850`
  License: MIT
  Notes: repository-local copy of the upstream AppleScript grammar, scanner and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-applescript/vendor/applescript/.parser-inputs`
- `crates/tree-sitter-kat-applescript/vendor/applescript/grammar.json`
- `crates/tree-sitter-kat-applescript/vendor/applescript/node-types.json`
- `crates/tree-sitter-kat-applescript/vendor/applescript/parser.c`
- `crates/tree-sitter-kat-applescript/vendor/applescript/scanner.c`

## asciidoc

Imported source assets:
- `upstreams/asciidoc/common/grammar.js`
- `upstreams/asciidoc/grammar.js`
- `upstreams/asciidoc/scanner.c`
- `upstreams/asciidoc/tree_sitter_asciidoc/chars.c`
- `upstreams/asciidoc/tree_sitter_asciidoc/chars.h`
- `upstreams/asciidoc/tree_sitter_asciidoc/marker_types.h`
- `upstreams/asciidoc/tree_sitter_asciidoc/markers.h`
- `upstreams/asciidoc/tree_sitter_asciidoc/scanner.c`
- `upstreams/asciidoc/tree_sitter_asciidoc/scanner.h`
- `upstreams/asciidoc/tree_sitter_asciidoc/tokens.h`

Original provenance carried over from `kat`:
Source: [cpkio/tree-sitter-asciidoc](https://github.com/cpkio/tree-sitter-asciidoc)
  Revision: `a00a91dd44cd6c228f3bc10b3e548f651058e0db`
  License: MIT
  Notes: repository-local copy of the upstream AsciiDoc grammar, scanner, required support files and highlights query. The vendored `tree_sitter_asciidoc/*` support sources are kept because the scanner depends on them at build time.

Local generated artifacts:
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/.parser-inputs`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/grammar.json`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/node-types.json`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/parser.c`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/scanner.c`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/chars.c`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/chars.h`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/marker_types.h`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/markers.h`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/scanner.c`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/scanner.h`
- `crates/tree-sitter-kat-asciidoc/vendor/asciidoc/tree_sitter_asciidoc/tokens.h`

## asm

Imported source assets:
- `upstreams/asm/grammar.js`

Original provenance carried over from `kat`:
Source: [RubixDev/tree-sitter-asm](https://github.com/RubixDev/tree-sitter-asm)
  Revision: `839741fef4dab5128952334624905c82b40c7133`
  License: MIT
  Notes: repository-local copy of the upstream assembly grammar and highlights query, currently used for `.s` / `.S` source files.

Local generated artifacts:
- `crates/tree-sitter-kat-asm/vendor/asm/.parser-inputs`
- `crates/tree-sitter-kat-asm/vendor/asm/grammar.json`
- `crates/tree-sitter-kat-asm/vendor/asm/node-types.json`
- `crates/tree-sitter-kat-asm/vendor/asm/parser.c`

## authorized_keys

Imported source assets:
- `upstreams/authorized_keys/grammar.js`

Original provenance carried over from `kat`:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `authorized_keys` / `.pub` style SSH public key files.

Local generated artifacts:
- `crates/tree-sitter-kat-authorized-keys/vendor/authorized_keys/.parser-inputs`
- `crates/tree-sitter-kat-authorized-keys/vendor/authorized_keys/grammar.json`
- `crates/tree-sitter-kat-authorized-keys/vendor/authorized_keys/node-types.json`
- `crates/tree-sitter-kat-authorized-keys/vendor/authorized_keys/parser.c`

## awk

Imported source assets:
- `upstreams/awk/grammar.js`
- `upstreams/awk/scanner.c`

Original provenance carried over from `kat`:
Source: [Beaglefoot/tree-sitter-awk](https://github.com/Beaglefoot/tree-sitter-awk)
  Revision: `34bbdc7cce8e803096f47b625979e34c1be38127`
  License: MIT
  Notes: repository-local copy of the upstream AWK grammar, scanner and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-awk/vendor/awk/.parser-inputs`
- `crates/tree-sitter-kat-awk/vendor/awk/grammar.json`
- `crates/tree-sitter-kat-awk/vendor/awk/node-types.json`
- `crates/tree-sitter-kat-awk/vendor/awk/parser.c`
- `crates/tree-sitter-kat-awk/vendor/awk/scanner.c`

## bibtex

Imported source assets:
- `upstreams/bibtex/grammar.js`

Original provenance carried over from `kat`:
Source: [latex-lsp/tree-sitter-bibtex](https://github.com/latex-lsp/tree-sitter-bibtex)
  Revision: `8d04ed27b3bc7929f14b7df9236797dab9f3fa66`
  License: MIT
  Notes: repository-local copy of the upstream BibTeX grammar, highlights query and locals query.

Local generated artifacts:
- `crates/tree-sitter-kat-bibtex/vendor/bibtex/.parser-inputs`
- `crates/tree-sitter-kat-bibtex/vendor/bibtex/grammar.json`
- `crates/tree-sitter-kat-bibtex/vendor/bibtex/node-types.json`
- `crates/tree-sitter-kat-bibtex/vendor/bibtex/parser.c`

## cabal

Imported source assets:
- `upstreams/cabal/grammar.js`
- `upstreams/cabal/scanner.c`

Original provenance carried over from `kat`:
Source: [thomasvergne/tree-sitter-cabal](https://github.com/thomasvergne/tree-sitter-cabal)
  Revision: `1762ded13e5351c0bc662a2273d523b80d314b4e`
  License: MIT
  Notes: repository-local copy of the upstream Cabal grammar and scanner, plus a kat-local highlights query maintained against that AST.

Local generated artifacts:
- `crates/tree-sitter-kat-cabal/vendor/cabal/.parser-inputs`
- `crates/tree-sitter-kat-cabal/vendor/cabal/grammar.json`
- `crates/tree-sitter-kat-cabal/vendor/cabal/node-types.json`
- `crates/tree-sitter-kat-cabal/vendor/cabal/parser.c`
- `crates/tree-sitter-kat-cabal/vendor/cabal/scanner.c`

## cmakecache

Imported source assets:
- `upstreams/cmakecache/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `CMakeCache.txt`.

Local generated artifacts:
- `crates/tree-sitter-kat-cmakecache/vendor/cmakecache/.parser-inputs`
- `crates/tree-sitter-kat-cmakecache/vendor/cmakecache/grammar.json`
- `crates/tree-sitter-kat-cmakecache/vendor/cmakecache/node-types.json`
- `crates/tree-sitter-kat-cmakecache/vendor/cmakecache/parser.c`

## command_help

Imported source assets:
- `upstreams/command_help/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for command help text files.

Local generated artifacts:
- `crates/tree-sitter-kat-command-help/vendor/command_help/.parser-inputs`
- `crates/tree-sitter-kat-command-help/vendor/command_help/grammar.json`
- `crates/tree-sitter-kat-command-help/vendor/command_help/node-types.json`
- `crates/tree-sitter-kat-command-help/vendor/command_help/parser.c`

## cpuinfo

Imported source assets:
- `upstreams/cpuinfo/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `cpuinfo`-style key/value files.

Local generated artifacts:
- `crates/tree-sitter-kat-cpuinfo/vendor/cpuinfo/.parser-inputs`
- `crates/tree-sitter-kat-cpuinfo/vendor/cpuinfo/grammar.json`
- `crates/tree-sitter-kat-cpuinfo/vendor/cpuinfo/node-types.json`
- `crates/tree-sitter-kat-cpuinfo/vendor/cpuinfo/parser.c`

## crontab

Imported source assets:
- `upstreams/crontab/grammar.js`

Original provenance carried over from `kat`:
Source: [slqy123/tree-sitter-crontab](https://github.com/slqy123/tree-sitter-crontab)
  Revision: `70b5628278756c3dc429fac6545fe7b2e8c553a0`
  License: MIT
  Notes: repository-local copy of the upstream crontab grammar and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-crontab/vendor/crontab/.parser-inputs`
- `crates/tree-sitter-kat-crontab/vendor/crontab/grammar.json`
- `crates/tree-sitter-kat-crontab/vendor/crontab/node-types.json`
- `crates/tree-sitter-kat-crontab/vendor/crontab/parser.c`

## css

Imported source assets:
- `upstreams/css/grammar.js`
- `upstreams/css/scanner.c`

Original provenance carried over from `kat`:
Block 1:
Source: [tree-sitter/tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)
  Revision: `4a9aab1668bf13d024710420648ef9a9ee6ccc17`
  License: MIT
  Notes: repository-local copy of the upstream grammar source, retained because the vendored Less and SCSS grammars import this file during their own build step. CSS parser sources are otherwise no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-css](https://crates.io/crates/tree-sitter-css).

Block 2:
Source: [zed-industries/zed](https://github.com/zed-industries/zed)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted copy of Zed's CSS highlights query. CSS parser sources are no longer vendored in this repository; the runtime parser now comes from the Rust crate [tree-sitter-css](https://crates.io/crates/tree-sitter-css).

Local generated artifacts:
- `crates/tree-sitter-kat-css/vendor/css/.parser-inputs`
- `crates/tree-sitter-kat-css/vendor/css/grammar.json`
- `crates/tree-sitter-kat-css/vendor/css/node-types.json`
- `crates/tree-sitter-kat-css/vendor/css/parser.c`
- `crates/tree-sitter-kat-css/vendor/css/scanner.c`

## csv

Imported source assets:
- `upstreams/csv/common/define-grammar.js`
- `upstreams/csv/grammar.js`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-csv](https://github.com/tree-sitter-grammars/tree-sitter-csv)
  Revision: `f6bf6e35eb0b95fbadea4bb39cb9709507fcb181`
  License: MIT
  Notes: repository-local extracted CSV subset built from the upstream separated-values grammar support file, with a kat-local wrapper kept as a dedicated runtime for comma-separated data files.

Local generated artifacts:
- `crates/tree-sitter-kat-csv/vendor/csv/.parser-inputs`
- `crates/tree-sitter-kat-csv/vendor/csv/grammar.json`
- `crates/tree-sitter-kat-csv/vendor/csv/node-types.json`
- `crates/tree-sitter-kat-csv/vendor/csv/parser.c`

## debsources

Imported source assets:
- `upstreams/debsources/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for Debian `sources.list` files.

Local generated artifacts:
- `crates/tree-sitter-kat-debsources/vendor/debsources/.parser-inputs`
- `crates/tree-sitter-kat-debsources/vendor/debsources/grammar.json`
- `crates/tree-sitter-kat-debsources/vendor/debsources/node-types.json`
- `crates/tree-sitter-kat-debsources/vendor/debsources/parser.c`

## dockerfile

Imported source assets:
- `upstreams/dockerfile/grammar.js`
- `upstreams/dockerfile/scanner.c`

Original provenance carried over from `kat`:
Block 1:
Source: [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  Revision: `971acdd908568b4531b0ba28a445bf0bb720aba5`
  License: MIT
  Notes: repository-local adapted copy of the upstream Dockerfile grammar/scanner/highlights.

Block 2:
Sources:
  - [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)
  - [tree-sitter/tree-sitter-bash](https://github.com/tree-sitter/tree-sitter-bash)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local injection query that routes Dockerfile shell-form instruction bodies into kat's shared Bash runtime. The exact upstream commits used as the starting point still need a later audit.

Local generated artifacts:
- `crates/tree-sitter-kat-dockerfile/vendor/dockerfile/.parser-inputs`
- `crates/tree-sitter-kat-dockerfile/vendor/dockerfile/grammar.json`
- `crates/tree-sitter-kat-dockerfile/vendor/dockerfile/node-types.json`
- `crates/tree-sitter-kat-dockerfile/vendor/dockerfile/parser.c`
- `crates/tree-sitter-kat-dockerfile/vendor/dockerfile/scanner.c`

## dot

Imported source assets:
- `upstreams/dot/grammar.js`

Original provenance carried over from `kat`:
Source: [rydesun/tree-sitter-dot](https://github.com/rydesun/tree-sitter-dot)
  Revision: `80327abbba6f47530edeb0df9f11bd5d5c93c14d`
  License: MIT
  Notes: repository-local copy of the upstream Graphviz DOT grammar plus highlights / injections queries.

Local generated artifacts:
- `crates/tree-sitter-kat-dot/vendor/dot/.parser-inputs`
- `crates/tree-sitter-kat-dot/vendor/dot/grammar.json`
- `crates/tree-sitter-kat-dot/vendor/dot/node-types.json`
- `crates/tree-sitter-kat-dot/vendor/dot/parser.c`

## dotenv

Imported source assets:
- `upstreams/dotenv/grammar.js`
- `upstreams/dotenv/scanner.c`

Original provenance carried over from `kat`:
Source: [pnx/tree-sitter-dotenv](https://github.com/pnx/tree-sitter-dotenv)
  Revision: `f3b1f1f20d255082f2fd4761f6961ab5cf01d4f4`
  License: MIT
  Notes: repository-local copy of the upstream DotENV grammar, highlights query and scanner.

Local generated artifacts:
- `crates/tree-sitter-kat-dotenv/vendor/dotenv/.parser-inputs`
- `crates/tree-sitter-kat-dotenv/vendor/dotenv/grammar.json`
- `crates/tree-sitter-kat-dotenv/vendor/dotenv/node-types.json`
- `crates/tree-sitter-kat-dotenv/vendor/dotenv/parser.c`
- `crates/tree-sitter-kat-dotenv/vendor/dotenv/scanner.c`

## email

Imported source assets:
- `upstreams/email/grammar.js`

Original provenance carried over from `kat`:
Source: [stevenxxiu/tree-sitter-mail](https://github.com/stevenxxiu/tree-sitter-mail)
  Revision: `8d2905d06a15586652c3a73387b4170424201e1a`
  License: MIT
  Notes: repository-local copy of the upstream mail grammar and highlights query, adapted to the repository build pipeline.

Local generated artifacts:
- `crates/tree-sitter-kat-email/vendor/email/.parser-inputs`
- `crates/tree-sitter-kat-email/vendor/email/grammar.json`
- `crates/tree-sitter-kat-email/vendor/email/node-types.json`
- `crates/tree-sitter-kat-email/vendor/email/parser.c`

## fish

Imported source assets:
- `upstreams/fish/grammar.js`
- `upstreams/fish/scanner.c`

Original provenance carried over from `kat`:
Source: [ram02z/tree-sitter-fish](https://github.com/ram02z/tree-sitter-fish)
  Revision: `fa2143f5d66a9eb6c007ba9173525ea7aaafe788`
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored.

Local generated artifacts:
- `crates/tree-sitter-kat-fish/vendor/fish/.parser-inputs`
- `crates/tree-sitter-kat-fish/vendor/fish/grammar.json`
- `crates/tree-sitter-kat-fish/vendor/fish/node-types.json`
- `crates/tree-sitter-kat-fish/vendor/fish/parser.c`
- `crates/tree-sitter-kat-fish/vendor/fish/scanner.c`

## fortran_namelist

Imported source assets:
- `upstreams/fortran_namelist/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for Fortran namelist files.

Local generated artifacts:
- `crates/tree-sitter-kat-fortran-namelist/vendor/fortran_namelist/.parser-inputs`
- `crates/tree-sitter-kat-fortran-namelist/vendor/fortran_namelist/grammar.json`
- `crates/tree-sitter-kat-fortran-namelist/vendor/fortran_namelist/node-types.json`
- `crates/tree-sitter-kat-fortran-namelist/vendor/fortran_namelist/parser.c`

## fstab

Imported source assets:
- `upstreams/fstab/grammar.js`

Original provenance carried over from `kat`:
Source: repository-local
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `fstab` / `crypttab` / `mtab` style files.

Local generated artifacts:
- `crates/tree-sitter-kat-fstab/vendor/fstab/.parser-inputs`
- `crates/tree-sitter-kat-fstab/vendor/fstab/grammar.json`
- `crates/tree-sitter-kat-fstab/vendor/fstab/node-types.json`
- `crates/tree-sitter-kat-fstab/vendor/fstab/parser.c`

## git_commit

Imported source assets:
- `upstreams/git_commit/grammar.js`

Original provenance carried over from `kat`:
Source: [the-mikedavis/tree-sitter-git-commit](https://github.com/the-mikedavis/tree-sitter-git-commit)
  Revision: `5a50da19b3841ac51e9d483cd9c856a85232233d`
  License: MIT
  Notes: repository-local copy of the upstream git commit grammar, with highlights query adapted to kat's capture model.

Local generated artifacts:
- `crates/tree-sitter-kat-git-commit/vendor/git_commit/.parser-inputs`
- `crates/tree-sitter-kat-git-commit/vendor/git_commit/grammar.json`
- `crates/tree-sitter-kat-git-commit/vendor/git_commit/node-types.json`
- `crates/tree-sitter-kat-git-commit/vendor/git_commit/parser.c`

## git_config

Imported source assets:
- `upstreams/git_config/grammar.js`

Original provenance carried over from `kat`:
Source: [the-mikedavis/tree-sitter-git-config](https://github.com/the-mikedavis/tree-sitter-git-config)
  Revision: `0fbc9f99d5a28865f9de8427fb0672d66f9d83a5`
  License: MIT
  Notes: repository-local copy of the upstream Git config grammar and highlights query, kept as a dedicated runtime so `.gitconfig`, `.gitmodules`, `.git/config` and related paths can share one parser while preserving room for future Git-specific profile overlays.

Local generated artifacts:
- `crates/tree-sitter-kat-git-config/vendor/git_config/.parser-inputs`
- `crates/tree-sitter-kat-git-config/vendor/git_config/grammar.json`
- `crates/tree-sitter-kat-git-config/vendor/git_config/node-types.json`
- `crates/tree-sitter-kat-git-config/vendor/git_config/parser.c`

## git_link

Imported source assets:
- `upstreams/git_link/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-git-link/vendor/git_link/.parser-inputs`
- `crates/tree-sitter-kat-git-link/vendor/git_link/grammar.json`
- `crates/tree-sitter-kat-git-link/vendor/git_link/node-types.json`
- `crates/tree-sitter-kat-git-link/vendor/git_link/parser.c`

## git_log

Imported source assets:
- `upstreams/git_log/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-git-log/vendor/git_log/.parser-inputs`
- `crates/tree-sitter-kat-git-log/vendor/git_log/grammar.json`
- `crates/tree-sitter-kat-git-log/vendor/git_log/node-types.json`
- `crates/tree-sitter-kat-git-log/vendor/git_log/parser.c`

## git_mailmap

Imported source assets:
- `upstreams/git_mailmap/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-git-mailmap/vendor/git_mailmap/.parser-inputs`
- `crates/tree-sitter-kat-git-mailmap/vendor/git_mailmap/grammar.json`
- `crates/tree-sitter-kat-git-mailmap/vendor/git_mailmap/node-types.json`
- `crates/tree-sitter-kat-git-mailmap/vendor/git_mailmap/parser.c`

## git_rebase

Imported source assets:
- `upstreams/git_rebase/grammar.js`

Original provenance carried over from `kat`:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `git-rebase-todo`.

Local generated artifacts:
- `crates/tree-sitter-kat-git-rebase/vendor/git_rebase/.parser-inputs`
- `crates/tree-sitter-kat-git-rebase/vendor/git_rebase/grammar.json`
- `crates/tree-sitter-kat-git-rebase/vendor/git_rebase/node-types.json`
- `crates/tree-sitter-kat-git-rebase/vendor/git_rebase/parser.c`

## gitattributes

Imported source assets:
- `upstreams/gitattributes/grammar.js`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-gitattributes](https://github.com/tree-sitter-grammars/tree-sitter-gitattributes)
  Revision: `1b7af09d45b579f9f288453b95ad555f1f431645`
  License: MIT
  Notes: repository-local copy of the upstream gitattributes grammar and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-gitattributes/vendor/gitattributes/.parser-inputs`
- `crates/tree-sitter-kat-gitattributes/vendor/gitattributes/grammar.json`
- `crates/tree-sitter-kat-gitattributes/vendor/gitattributes/node-types.json`
- `crates/tree-sitter-kat-gitattributes/vendor/gitattributes/parser.c`

## gomod

Imported source assets:
- `upstreams/gomod/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [camdencheek/tree-sitter-go-mod](https://github.com/camdencheek/tree-sitter-go-mod)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source for `go.mod`. The exact upstream commit for the initial import still needs a later audit.

Block 2:
Source: [camdencheek/tree-sitter-go-mod](https://github.com/camdencheek/tree-sitter-go-mod)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted highlights query for `go.mod`, further refined for kat's terminal rendering.

Local generated artifacts:
- `crates/tree-sitter-kat-gomod/vendor/gomod/.parser-inputs`
- `crates/tree-sitter-kat-gomod/vendor/gomod/grammar.json`
- `crates/tree-sitter-kat-gomod/vendor/gomod/node-types.json`
- `crates/tree-sitter-kat-gomod/vendor/gomod/parser.c`

## gosum

Imported source assets:
- `upstreams/gosum/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [amaanq/tree-sitter-go-sum](https://github.com/amaanq/tree-sitter-go-sum)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source for `go.sum`. The exact upstream commit for the initial import still needs a later audit.

Block 2:
Source: [amaanq/tree-sitter-go-sum](https://github.com/amaanq/tree-sitter-go-sum)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted highlights query for `go.sum`, further refined for kat's terminal rendering.

Local generated artifacts:
- `crates/tree-sitter-kat-gosum/vendor/gosum/.parser-inputs`
- `crates/tree-sitter-kat-gosum/vendor/gosum/grammar.json`
- `crates/tree-sitter-kat-gosum/vendor/gosum/node-types.json`
- `crates/tree-sitter-kat-gosum/vendor/gosum/parser.c`

## gowork

Imported source assets:
- `upstreams/gowork/grammar.js`

Original provenance carried over from `kat`:
Sources:
  - [omertuc/tree-sitter-go-work](https://github.com/omertuc/tree-sitter-go-work)
  - [Go Modules Reference](https://go.dev/ref/mod#workspaces)
  Revisions: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted copy of the upstream grammar/query, kept as a dedicated runtime for `go.work` instead of mixing workspace files into the Go source runtime. The exact upstream commit for the grammar/query starting point still needs a later audit.

Local generated artifacts:
- `crates/tree-sitter-kat-gowork/vendor/gowork/.parser-inputs`
- `crates/tree-sitter-kat-gowork/vendor/gowork/grammar.json`
- `crates/tree-sitter-kat-gowork/vendor/gowork/node-types.json`
- `crates/tree-sitter-kat-gowork/vendor/gowork/parser.c`

## graphql

Imported source assets:
- `upstreams/graphql/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [joowani/tree-sitter-graphql](https://github.com/joowani/tree-sitter-graphql)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local adapted grammar source. The exact upstream commit for the initial import still needs a later audit.

Block 2:
Source: repository-local query maintained against the node structure of [joowani/tree-sitter-graphql](https://github.com/joowani/tree-sitter-graphql)
  Revision: not recorded during the initial import
  License: MIT
  Notes: highlights query maintained in-repo and tuned for kat's renderer/runtime model.

Local generated artifacts:
- `crates/tree-sitter-kat-graphql/vendor/graphql/.parser-inputs`
- `crates/tree-sitter-kat-graphql/vendor/graphql/grammar.json`
- `crates/tree-sitter-kat-graphql/vendor/graphql/node-types.json`
- `crates/tree-sitter-kat-graphql/vendor/graphql/parser.c`

## hcl

Imported source assets:
- `upstreams/hcl/grammar.js`
- `upstreams/hcl/scanner.c`

Original provenance carried over from `kat`:
Block 1:
Source: [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl)
  Revision: `fad991865fee927dd1de5e172fb3f08ac674d914`
  License: Apache-2.0
  Notes: repository-local adapted copy of the upstream HCL grammar/scanner, kept as a dedicated runtime for `.hcl` and `.nomad` files.

Block 2:
Sources:
  - [helix-editor/helix](https://github.com/helix-editor/helix/blob/035450a2de62142b4117c01b89fff3d4f1b4d51f/runtime/queries/hcl/highlights.scm)
  - [tree-sitter-grammars/tree-sitter-hcl](https://github.com/tree-sitter-grammars/tree-sitter-hcl)
  License: MPL-2.0 and Apache-2.0
  Notes: repository-local adapted HCL highlights query, starting from Helix's query and tuned for kat's capture/theme model.

Local generated artifacts:
- `crates/tree-sitter-kat-hcl/vendor/hcl/.parser-inputs`
- `crates/tree-sitter-kat-hcl/vendor/hcl/grammar.json`
- `crates/tree-sitter-kat-hcl/vendor/hcl/node-types.json`
- `crates/tree-sitter-kat-hcl/vendor/hcl/parser.c`
- `crates/tree-sitter-kat-hcl/vendor/hcl/scanner.c`

## ignore

Imported source assets:
- `upstreams/ignore/grammar.js`

Original provenance carried over from `kat`:
Source: [shunsambongi/tree-sitter-gitignore](https://github.com/shunsambongi/tree-sitter-gitignore)
  Revision: `f4685bf11ac466dd278449bcfe5fd014e94aa504`
  License: MIT
  Notes: repository-local adapted copy of the upstream ignore-pattern grammar, renamed to `ignore` so `.gitignore`, `.dockerignore`, `.npmignore` and similar files can share one runtime.

Local generated artifacts:
- `crates/tree-sitter-kat-ignore/vendor/ignore/.parser-inputs`
- `crates/tree-sitter-kat-ignore/vendor/ignore/grammar.json`
- `crates/tree-sitter-kat-ignore/vendor/ignore/node-types.json`
- `crates/tree-sitter-kat-ignore/vendor/ignore/parser.c`

## jinja

Imported source assets:
- `upstreams/jinja/base_grammar.js`
- `upstreams/jinja/common/common.js`
- `upstreams/jinja/common/expression.js`
- `upstreams/jinja/common/literal.js`
- `upstreams/jinja/grammar.js`
- `upstreams/jinja/scanner.c`

Original provenance carried over from `kat`:
Source: [cathaysia/tree-sitter-jinja](https://github.com/cathaysia/tree-sitter-jinja)
  Revision: `7e254abb76618227806f6881525980231faa1610`
  License: Apache-2.0
  Notes: repository-local adapted copy of the upstream Jinja grammar and support files. `grammars/jinja/grammar.js` is a kat-local wrapper that points at the vendored base grammar file layout. Template host dispatch is now handled by kat's document-profile + host-resolver layer instead of a dedicated Jinja injections query.

Local generated artifacts:
- `crates/tree-sitter-kat-jinja/vendor/jinja/.parser-inputs`
- `crates/tree-sitter-kat-jinja/vendor/jinja/grammar.json`
- `crates/tree-sitter-kat-jinja/vendor/jinja/node-types.json`
- `crates/tree-sitter-kat-jinja/vendor/jinja/parser.c`
- `crates/tree-sitter-kat-jinja/vendor/jinja/scanner.c`

## jq

Imported source assets:
- `upstreams/jq/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [flurie/tree-sitter-jq](https://github.com/flurie/tree-sitter-jq)
  Revision: `c204e36d2c3c6fce1f57950b12cabcc24e5cc4d9`
  License: BSD-3-Clause
  Notes: repository-local copy of the upstream jq grammar. This vendor source is used instead of the GPL-licensed `nverno/tree-sitter-jq` line so kat can keep a permissive third-party baseline.

Block 2:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local jq highlights query written for kat against the vendored BSD jq grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-jq/vendor/jq/.parser-inputs`
- `crates/tree-sitter-kat-jq/vendor/jq/grammar.json`
- `crates/tree-sitter-kat-jq/vendor/jq/node-types.json`
- `crates/tree-sitter-kat-jq/vendor/jq/parser.c`

## just

Imported source assets:
- `upstreams/just/grammar.js`
- `upstreams/just/scanner.c`

Original provenance carried over from `kat`:
Block 1:
Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: `d9da862c156020c1a83d3c6ccdda32be6d8a5d4a`
  License: Apache-2.0
  Notes: repository-local copy of the upstream grammar source.

Block 2:
Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local copy of the upstream scanner source. The exact upstream commit for the initial import still needs a later audit.

Block 3:
Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: `7333f8c150aaac5bb46decc2d225a2d4cde8c51e`
  License: Apache-2.0
  Notes: repository-local copy of the upstream generated highlights query.

Block 4:
Source: [IndianBoy42/tree-sitter-just](https://github.com/IndianBoy42/tree-sitter-just)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local adapted injections query. The vendored file is not currently pinned to a verified upstream commit.

Local generated artifacts:
- `crates/tree-sitter-kat-just/vendor/just/.parser-inputs`
- `crates/tree-sitter-kat-just/vendor/just/grammar.json`
- `crates/tree-sitter-kat-just/vendor/just/node-types.json`
- `crates/tree-sitter-kat-just/vendor/just/parser.c`
- `crates/tree-sitter-kat-just/vendor/just/scanner.c`

## latex

Imported source assets:
- `upstreams/latex/grammar.js`
- `upstreams/latex/scanner.c`

Original provenance carried over from `kat`:
Block 1:
Source: [latex-lsp/tree-sitter-latex](https://github.com/latex-lsp/tree-sitter-latex)
  Revision: `7e0ecdc02926c7b9b2e0c76003d4fe7b0944f957`
  License: MIT
  Notes: repository-local copy of the upstream LaTeX grammar and scanner, kept as a dedicated runtime for `.tex` / `.ltx` / `.sty` / `.cls`.

Block 2:
Source: [nvim-treesitter/nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)
  Revision: `cf12346a3414fa1b06af75c79faebe7f76df080a`
  License: Apache-2.0
  Notes: repository-local adapted LaTeX query assets based on nvim-treesitter, retuned for kat's capture model and nested-runtime integration. The injections query keeps high-value `minted` / `pycode` / `luacode` style environments routed into kat's shared runtimes.

Local generated artifacts:
- `crates/tree-sitter-kat-latex/vendor/latex/.parser-inputs`
- `crates/tree-sitter-kat-latex/vendor/latex/grammar.json`
- `crates/tree-sitter-kat-latex/vendor/latex/node-types.json`
- `crates/tree-sitter-kat-latex/vendor/latex/parser.c`
- `crates/tree-sitter-kat-latex/vendor/latex/scanner.c`

## less

Imported source assets:
- `upstreams/less/grammar.js`
- `upstreams/less/scanner.c`

Original provenance carried over from `kat`:
Source: [amaanq/tree-sitter-less](https://github.com/amaanq/tree-sitter-less)
  Revision: `e5ae6245f841b5778c79ac93b28fa4f56b679c5d`
  License: MIT
  Notes: repository-local copy of the upstream Less grammar, scanner and highlights query. `grammars/less/grammar.js` is locally adjusted to reuse kat's vendored CSS grammar instead of depending on a second external CSS package.

Local generated artifacts:
- `crates/tree-sitter-kat-less/vendor/less/.parser-inputs`
- `crates/tree-sitter-kat-less/vendor/less/grammar.json`
- `crates/tree-sitter-kat-less/vendor/less/node-types.json`
- `crates/tree-sitter-kat-less/vendor/less/parser.c`
- `crates/tree-sitter-kat-less/vendor/less/scanner.c`

## markdown

Imported source assets:
- `upstreams/markdown/common.js`
- `upstreams/markdown/grammar.js`
- `upstreams/markdown/html_entities.json`
- `upstreams/markdown/scanner.c`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-markdown](https://github.com/tree-sitter-grammars/tree-sitter-markdown)
  Revision: not recorded during the initial import
  License: MIT
  Notes: repository-local copy of selected grammar sources only; generated parser artifacts are not stored. The exact upstream commit for the initial import still needs a later audit. `grammars/markdown/common.js` and `grammars/markdown/html_entities.json` are copied from the upstream shared support files because the block and inline Markdown grammars depend on them at build time.

Local generated artifacts:
- `crates/tree-sitter-kat-markdown/vendor/markdown/.parser-inputs`
- `crates/tree-sitter-kat-markdown/vendor/markdown/grammar.json`
- `crates/tree-sitter-kat-markdown/vendor/markdown/node-types.json`
- `crates/tree-sitter-kat-markdown/vendor/markdown/parser.c`
- `crates/tree-sitter-kat-markdown/vendor/markdown/scanner.c`

## markdown_inline

Imported source assets:
- `upstreams/markdown_inline/grammar.js`
- `upstreams/markdown_inline/scanner.c`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-markdown-inline/vendor/markdown_inline/.parser-inputs`
- `crates/tree-sitter-kat-markdown-inline/vendor/markdown_inline/grammar.json`
- `crates/tree-sitter-kat-markdown-inline/vendor/markdown_inline/node-types.json`
- `crates/tree-sitter-kat-markdown-inline/vendor/markdown_inline/parser.c`
- `crates/tree-sitter-kat-markdown-inline/vendor/markdown_inline/scanner.c`

## nasm

Imported source assets:
- `upstreams/nasm/grammar.js`

Original provenance carried over from `kat`:
Source: [naclsn/tree-sitter-nasm](https://github.com/naclsn/tree-sitter-nasm)
  Revision: `d1b3638d017f2a8585e26dcfc66fe1df94185e30`
  License: MIT
  Notes: repository-local copy of the upstream NASM grammar and highlights query, kept as a dedicated runtime for x86 / x86_64-flavored `.asm` / `.nasm` / `.yasm` / `.inc` / `.mac` sources.

Local generated artifacts:
- `crates/tree-sitter-kat-nasm/vendor/nasm/.parser-inputs`
- `crates/tree-sitter-kat-nasm/vendor/nasm/grammar.json`
- `crates/tree-sitter-kat-nasm/vendor/nasm/node-types.json`
- `crates/tree-sitter-kat-nasm/vendor/nasm/parser.c`

## ninja

Imported source assets:
- `upstreams/ninja/grammar.js`

Original provenance carried over from `kat`:
Source: [alemuller/tree-sitter-ninja](https://github.com/alemuller/tree-sitter-ninja)
  Revision: `0a95cfdc0745b6ae82f60d3a339b37f19b7b9267`
  License: MIT
  Notes: repository-local copy of the upstream Ninja grammar and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-ninja/vendor/ninja/.parser-inputs`
- `crates/tree-sitter-kat-ninja/vendor/ninja/grammar.json`
- `crates/tree-sitter-kat-ninja/vendor/ninja/node-types.json`
- `crates/tree-sitter-kat-ninja/vendor/ninja/parser.c`

## proto

Imported source assets:
- `upstreams/proto/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [mitchellh/tree-sitter-proto](https://github.com/mitchellh/tree-sitter-proto)
  Revision: `42d82fa18f8afe59b5fc0b16c207ee4f84cb185f`
  License: MIT
  Notes: repository-local copy of the upstream grammar source, with a minimal local JavaScript syntax fix applied so the vendored grammar can build correctly in kat's grammar pipeline.

Block 2:
Source: [mitchellh/tree-sitter-proto](https://github.com/mitchellh/tree-sitter-proto)
  Revision: `42d82fa18f8afe59b5fc0b16c207ee4f84cb185f`
  License: MIT
  Notes: repository-local adapted highlights query for kat's terminal renderer and capture model.

Local generated artifacts:
- `crates/tree-sitter-kat-proto/vendor/proto/.parser-inputs`
- `crates/tree-sitter-kat-proto/vendor/proto/grammar.json`
- `crates/tree-sitter-kat-proto/vendor/proto/node-types.json`
- `crates/tree-sitter-kat-proto/vendor/proto/parser.c`

## query

Imported source assets:
- `upstreams/query/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [nvim-treesitter/tree-sitter-query](https://github.com/nvim-treesitter/tree-sitter-query)
  Revision: `not recorded during the initial import`
  License: Apache-2.0
  Notes: vendored Tree-sitter query grammar source used by kat's local build pipeline.

Block 2:
Source: local integration query built around [nvim-treesitter/tree-sitter-query](https://github.com/nvim-treesitter/tree-sitter-query)
  Revision: not recorded during the initial import
  License: Apache-2.0
  Notes: repository-local highlights query for Tree-sitter query files, maintained in-repo alongside the vendored grammar source.

Local generated artifacts:
- `crates/tree-sitter-kat-query/vendor/query/.parser-inputs`
- `crates/tree-sitter-kat-query/vendor/query/grammar.json`
- `crates/tree-sitter-kat-query/vendor/query/node-types.json`
- `crates/tree-sitter-kat-query/vendor/query/parser.c`

## requirements

Imported source assets:
- `upstreams/requirements/grammar.js`

Original provenance carried over from `kat`:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for `requirements.txt` / `requirements.in` style files.

Local generated artifacts:
- `crates/tree-sitter-kat-requirements/vendor/requirements/.parser-inputs`
- `crates/tree-sitter-kat-requirements/vendor/requirements/grammar.json`
- `crates/tree-sitter-kat-requirements/vendor/requirements/node-types.json`
- `crates/tree-sitter-kat-requirements/vendor/requirements/parser.c`

## sass

Imported source assets:
- `upstreams/sass/grammar.js`
- `upstreams/sass/scanner.c`

Original provenance carried over from `kat`:
Source: [bajrangCoder/tree-sitter-sass](https://github.com/bajrangCoder/tree-sitter-sass)
  Revision: `fb280c41b070657e4ff4d4e5e6eea6cb19efd9b8`
  License: MIT
  Notes: repository-local copy of the upstream Sass grammar, scanner and highlights query.

Local generated artifacts:
- `crates/tree-sitter-kat-sass/vendor/sass/.parser-inputs`
- `crates/tree-sitter-kat-sass/vendor/sass/grammar.json`
- `crates/tree-sitter-kat-sass/vendor/sass/node-types.json`
- `crates/tree-sitter-kat-sass/vendor/sass/parser.c`
- `crates/tree-sitter-kat-sass/vendor/sass/scanner.c`

## scss

Imported source assets:
- `upstreams/scss/grammar.js`
- `upstreams/scss/scanner.c`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss)
  Revision: `2ef6d42e3ad7a8208900f9346f4529806ae0f9f9`
  License: MIT
  Notes: repository-local copy of the upstream SCSS grammar, scanner and highlights query. `grammars/scss/grammar.js` is locally adjusted to reuse kat's vendored CSS grammar instead of depending on a second external CSS package.

Local generated artifacts:
- `crates/tree-sitter-kat-scss/vendor/scss/.parser-inputs`
- `crates/tree-sitter-kat-scss/vendor/scss/grammar.json`
- `crates/tree-sitter-kat-scss/vendor/scss/node-types.json`
- `crates/tree-sitter-kat-scss/vendor/scss/parser.c`
- `crates/tree-sitter-kat-scss/vendor/scss/scanner.c`

## sml

Imported source assets:
- `upstreams/sml/grammar.js`
- `upstreams/sml/scanner.c`

Original provenance carried over from `kat`:
Source: [matthew-fluet/tree-sitter-sml](https://github.com/matthew-fluet/tree-sitter-sml)
  Revision: `fd4b4955bb998262840ab8119885b3edf20ea75a`
  License: MIT
  Notes: repository-local copy of the upstream SML grammar, scanner and highlights query, kept as a dedicated runtime for `.sml` / `.cm` / `.sig`.

Local generated artifacts:
- `crates/tree-sitter-kat-sml/vendor/sml/.parser-inputs`
- `crates/tree-sitter-kat-sml/vendor/sml/grammar.json`
- `crates/tree-sitter-kat-sml/vendor/sml/node-types.json`
- `crates/tree-sitter-kat-sml/vendor/sml/parser.c`
- `crates/tree-sitter-kat-sml/vendor/sml/scanner.c`

## ssh_config

Imported source assets:
- `upstreams/ssh_config/grammar.js`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-ssh-config](https://github.com/tree-sitter-grammars/tree-sitter-ssh-config)
  Revision: `71d2693deadaca8cdc09e38ba41d2f6042da1616`
  License: MIT
  Notes: repository-local copy of the upstream OpenSSH config grammar and query assets.

Local generated artifacts:
- `crates/tree-sitter-kat-ssh-config/vendor/ssh_config/.parser-inputs`
- `crates/tree-sitter-kat-ssh-config/vendor/ssh_config/grammar.json`
- `crates/tree-sitter-kat-ssh-config/vendor/ssh_config/node-types.json`
- `crates/tree-sitter-kat-ssh-config/vendor/ssh_config/parser.c`

## strace

Imported source assets:
- `upstreams/strace/grammar.js`

Original provenance carried over from `kat`:
Source: [sigmaSd/tree-sitter-strace](https://github.com/sigmaSd/tree-sitter-strace)
  Revision: `ac874ddfcc08d689fee1f4533789e06d88388f29`
  License: MIT
  Notes: repository-local copy of the upstream Strace grammar and highlights query, kept as a dedicated runtime for `strace` output files.

Local generated artifacts:
- `crates/tree-sitter-kat-strace/vendor/strace/.parser-inputs`
- `crates/tree-sitter-kat-strace/vendor/strace/grammar.json`
- `crates/tree-sitter-kat-strace/vendor/strace/node-types.json`
- `crates/tree-sitter-kat-strace/vendor/strace/parser.c`

## stylus

Imported source assets:
- `upstreams/stylus/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-stylus/vendor/stylus/.parser-inputs`
- `crates/tree-sitter-kat-stylus/vendor/stylus/grammar.json`
- `crates/tree-sitter-kat-stylus/vendor/stylus/node-types.json`
- `crates/tree-sitter-kat-stylus/vendor/stylus/parser.c`

## svelte

Imported source assets:
- `upstreams/svelte/allocator.h`
- `upstreams/svelte/ekstring.h`
- `upstreams/svelte/grammar.js`
- `upstreams/svelte/scanner.c`
- `upstreams/svelte/tag.h`
- `upstreams/svelte/uthash.h`
- `upstreams/svelte/vc_vector.h`

Original provenance carried over from `kat`:
Source: [Himujjal/tree-sitter-svelte](https://github.com/Himujjal/tree-sitter-svelte)
  Revision: `60ea1d673a1a3eeeb597e098d9ada9ed0c79ef4b`
  License: MIT
  Notes: repository-local copy of the upstream Svelte grammar, scanner and required support headers, plus kat-local highlights / injections queries.

Local generated artifacts:
- `crates/tree-sitter-kat-svelte/vendor/svelte/.parser-inputs`
- `crates/tree-sitter-kat-svelte/vendor/svelte/allocator.h`
- `crates/tree-sitter-kat-svelte/vendor/svelte/ekstring.h`
- `crates/tree-sitter-kat-svelte/vendor/svelte/grammar.json`
- `crates/tree-sitter-kat-svelte/vendor/svelte/node-types.json`
- `crates/tree-sitter-kat-svelte/vendor/svelte/parser.c`
- `crates/tree-sitter-kat-svelte/vendor/svelte/scanner.c`
- `crates/tree-sitter-kat-svelte/vendor/svelte/tag.h`
- `crates/tree-sitter-kat-svelte/vendor/svelte/uthash.h`
- `crates/tree-sitter-kat-svelte/vendor/svelte/vc_vector.h`

## syslog

Imported source assets:
- `upstreams/syslog/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-syslog/vendor/syslog/.parser-inputs`
- `crates/tree-sitter-kat-syslog/vendor/syslog/grammar.json`
- `crates/tree-sitter-kat-syslog/vendor/syslog/node-types.json`
- `crates/tree-sitter-kat-syslog/vendor/syslog/parser.c`

## tcl

Imported source assets:
- `upstreams/tcl/grammar.js`
- `upstreams/tcl/scanner.c`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-tcl](https://github.com/tree-sitter-grammars/tree-sitter-tcl)
  Revision: `8f11ac7206a54ed11210491cee1e0657e2962c47`
  License: MIT
  Notes: repository-local copy of the upstream Tcl grammar, scanner and highlights query, kept as a dedicated runtime for `.tcl` / `.tk` / `.tm` files and Tcl shebang scripts.

Local generated artifacts:
- `crates/tree-sitter-kat-tcl/vendor/tcl/.parser-inputs`
- `crates/tree-sitter-kat-tcl/vendor/tcl/grammar.json`
- `crates/tree-sitter-kat-tcl/vendor/tcl/node-types.json`
- `crates/tree-sitter-kat-tcl/vendor/tcl/parser.c`
- `crates/tree-sitter-kat-tcl/vendor/tcl/scanner.c`

## textile

Imported source assets:
- `upstreams/textile/grammar.js`

Original provenance carried over from `kat`:
Source: kat local integration
  Revision: n/a
  License: project-local
  Notes: repository-local minimal grammar and highlights query for Textile markup. An upstream Tree-sitter repository was evaluated, but it is not vendored here because its licensing metadata is currently unclear.

Local generated artifacts:
- `crates/tree-sitter-kat-textile/vendor/textile/.parser-inputs`
- `crates/tree-sitter-kat-textile/vendor/textile/grammar.json`
- `crates/tree-sitter-kat-textile/vendor/textile/node-types.json`
- `crates/tree-sitter-kat-textile/vendor/textile/parser.c`

## textproto

Imported source assets:
- `upstreams/textproto/grammar.js`

Original provenance carried over from `kat`:
Block 1:
Source: [PorterAtGoogle/tree-sitter-textproto](https://github.com/PorterAtGoogle/tree-sitter-textproto)
  Revision: `568471b80fd8793d37ed01865d8c2208a9fefd1b`
  License: MIT
  Notes: repository-local copy of the upstream grammar source.

Block 2:
Source: [PorterAtGoogle/tree-sitter-textproto](https://github.com/PorterAtGoogle/tree-sitter-textproto)
  Revision: `568471b80fd8793d37ed01865d8c2208a9fefd1b`
  License: MIT
  Notes: repository-local adapted highlights query for kat's capture model, including local semantic adjustments for field names and scalar literals.

Local generated artifacts:
- `crates/tree-sitter-kat-textproto/vendor/textproto/.parser-inputs`
- `crates/tree-sitter-kat-textproto/vendor/textproto/grammar.json`
- `crates/tree-sitter-kat-textproto/vendor/textproto/node-types.json`
- `crates/tree-sitter-kat-textproto/vendor/textproto/parser.c`

## todotxt

Imported source assets:
- `upstreams/todotxt/grammar.js`

Original provenance carried over from `kat`:
Source: [arnarg/tree-sitter-todotxt](https://github.com/arnarg/tree-sitter-todotxt)
  Revision: `3937c5cd105ec4127448651a21aef45f52d19609`
  License: MIT
  Notes: repository-local copy of the upstream todo.txt grammar and highlights query, kept as a dedicated runtime for `todo.txt` / `done.txt` style task lists.

Local generated artifacts:
- `crates/tree-sitter-kat-todotxt/vendor/todotxt/.parser-inputs`
- `crates/tree-sitter-kat-todotxt/vendor/todotxt/grammar.json`
- `crates/tree-sitter-kat-todotxt/vendor/todotxt/node-types.json`
- `crates/tree-sitter-kat-todotxt/vendor/todotxt/parser.c`

## tsv

Imported source assets:
- `upstreams/tsv/common/define-grammar.js`
- `upstreams/tsv/grammar.js`

Original provenance carried over from `kat`:
Source: [tree-sitter-grammars/tree-sitter-csv](https://github.com/tree-sitter-grammars/tree-sitter-csv)
  Revision: `f6bf6e35eb0b95fbadea4bb39cb9709507fcb181`
  License: MIT
  Notes: repository-local extracted TSV subset built from the upstream separated-values grammar support file, with a kat-local TSV wrapper and highlights query kept as a dedicated runtime for tab-separated data files.

Local generated artifacts:
- `crates/tree-sitter-kat-tsv/vendor/tsv/.parser-inputs`
- `crates/tree-sitter-kat-tsv/vendor/tsv/grammar.json`
- `crates/tree-sitter-kat-tsv/vendor/tsv/node-types.json`
- `crates/tree-sitter-kat-tsv/vendor/tsv/parser.c`

## twig

Imported source assets:
- `upstreams/twig/grammar.js`
- `upstreams/twig/scanner.c`

Original provenance carried over from `kat`:
Source: [kaermorchen/tree-sitter-twig](https://github.com/kaermorchen/tree-sitter-twig)
  Revision: `dac11024e40536d05c958d920139c310cbe86625`
  License: MPL-2.0
  Notes: repository-local copy of the upstream Twig grammar and scanner, with a kat-local highlights query layered on top. Template host dispatch is handled by kat's document-profile + host-resolver layer.

Local generated artifacts:
- `crates/tree-sitter-kat-twig/vendor/twig/.parser-inputs`
- `crates/tree-sitter-kat-twig/vendor/twig/grammar.json`
- `crates/tree-sitter-kat-twig/vendor/twig/node-types.json`
- `crates/tree-sitter-kat-twig/vendor/twig/parser.c`
- `crates/tree-sitter-kat-twig/vendor/twig/scanner.c`

## typst

Imported source assets:
- `upstreams/typst/grammar.js`
- `upstreams/typst/scanner.c`
- `upstreams/typst/unicode.h`

Original provenance carried over from `kat`:
Source: [uben0/tree-sitter-typst](https://github.com/uben0/tree-sitter-typst)
  Revision: `46cf4ded12ee974a70bf8457263b67ad7ee0379d`
  License: MIT
  Notes: repository-local copy of the upstream Typst grammar, scanner and required unicode support header, plus kat-local adapted highlights / injections queries for Typst markup and raw block nested runtimes.

Local generated artifacts:
- `crates/tree-sitter-kat-typst/vendor/typst/.parser-inputs`
- `crates/tree-sitter-kat-typst/vendor/typst/grammar.json`
- `crates/tree-sitter-kat-typst/vendor/typst/node-types.json`
- `crates/tree-sitter-kat-typst/vendor/typst/parser.c`
- `crates/tree-sitter-kat-typst/vendor/typst/scanner.c`
- `crates/tree-sitter-kat-typst/vendor/typst/unicode.h`

## userscript_metadata

Imported source assets:
- `upstreams/userscript_metadata/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-userscript-metadata/vendor/userscript_metadata/.parser-inputs`
- `crates/tree-sitter-kat-userscript-metadata/vendor/userscript_metadata/grammar.json`
- `crates/tree-sitter-kat-userscript-metadata/vendor/userscript_metadata/node-types.json`
- `crates/tree-sitter-kat-userscript-metadata/vendor/userscript_metadata/parser.c`

## vimhelp

Imported source assets:
- `upstreams/vimhelp/grammar.js`

Original provenance carried over from `kat`:
Source: [neovim/tree-sitter-vimdoc](https://github.com/neovim/tree-sitter-vimdoc)
  Revision: `f061895a0eff1d5b90e4fb60d21d87be3267031a`
  License: Apache-2.0
  Notes: repository-local copy of the upstream Vim help grammar and query assets, kept as a dedicated runtime for `.vimhelp` files.

Local generated artifacts:
- `crates/tree-sitter-kat-vimhelp/vendor/vimhelp/.parser-inputs`
- `crates/tree-sitter-kat-vimhelp/vendor/vimhelp/grammar.json`
- `crates/tree-sitter-kat-vimhelp/vendor/vimhelp/node-types.json`
- `crates/tree-sitter-kat-vimhelp/vendor/vimhelp/parser.c`

## vue

Imported source assets:
- `upstreams/vue/grammar.js`
- `upstreams/vue/scanner.cc`
- `upstreams/vue/tree_sitter_html/scanner.cc`
- `upstreams/vue/tree_sitter_html/tag.h`

Original provenance carried over from `kat`:
Source: [ikatyang/tree-sitter-vue](https://github.com/ikatyang/tree-sitter-vue)
  Revision: `91fe2754796cd8fba5f229505a23fa08f3546c06`
  License: MIT
  Notes: repository-local copy of the upstream Vue grammar and scanner support files, plus kat-local highlights / injections queries. `grammars/vue/scanner.cc` keeps the upstream logic but renames the embedded HTML scanner symbols so the dedicated Vue runtime can coexist with kat's standalone HTML runtime.

Local generated artifacts:
- `crates/tree-sitter-kat-vue/vendor/vue/.parser-inputs`
- `crates/tree-sitter-kat-vue/vendor/vue/grammar.json`
- `crates/tree-sitter-kat-vue/vendor/vue/node-types.json`
- `crates/tree-sitter-kat-vue/vendor/vue/parser.c`
- `crates/tree-sitter-kat-vue/vendor/vue/scanner.cc`
- `crates/tree-sitter-kat-vue/vendor/vue/tree_sitter_html/scanner.cc`
- `crates/tree-sitter-kat-vue/vendor/vue/tree_sitter_html/tag.h`

## vyper

Imported source assets:
- `upstreams/vyper/grammar.js`

Original provenance carried over from `kat`:
No dedicated provenance block was found in `kat/THIRD_PARTY.md`; see the repository history for this grammar.

Local generated artifacts:
- `crates/tree-sitter-kat-vyper/vendor/vyper/.parser-inputs`
- `crates/tree-sitter-kat-vyper/vendor/vyper/grammar.json`
- `crates/tree-sitter-kat-vyper/vendor/vyper/node-types.json`
- `crates/tree-sitter-kat-vyper/vendor/vyper/parser.c`

## wgsl

Imported source assets:
- `upstreams/wgsl/grammar.js`
- `upstreams/wgsl/scanner.c`

Original provenance carried over from `kat`:
Source: [szebniok/tree-sitter-wgsl](https://github.com/szebniok/tree-sitter-wgsl)
  Revision: `27ae4db1458d5ea7555300c9e93c6273723d420c`
  License: CC0-1.0
  Notes: repository-local copy of the upstream WGSL grammar and scanner, kept vendored because the published crate still depends on an older `tree-sitter` ABI. The highlights query is maintained locally for kat's capture model.

Local generated artifacts:
- `crates/tree-sitter-kat-wgsl/vendor/wgsl/.parser-inputs`
- `crates/tree-sitter-kat-wgsl/vendor/wgsl/grammar.json`
- `crates/tree-sitter-kat-wgsl/vendor/wgsl/node-types.json`
- `crates/tree-sitter-kat-wgsl/vendor/wgsl/parser.c`
- `crates/tree-sitter-kat-wgsl/vendor/wgsl/scanner.c`
