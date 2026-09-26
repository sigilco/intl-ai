# Dialect wordlists

Data backing the `dialect:<locale>` builtin check.

`en.json` maps American spellings to British ones (1714 entries). The
`dialect` check inverts it at startup, so `dialect:en-US` flags British
spellings and `dialect:en-GB` flags American ones.

## Provenance

Derived from `data/american_spellings.json` in the npm package
`american-british-english-translator` v0.2.1 by Kaz Sato
(hyperreality) and contributors:
https://github.com/hyperreality/American-British-English-Translator

MIT licensed — see LICENSE.
