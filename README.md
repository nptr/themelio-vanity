# themelio-vanity

A vanity wallet address generator for Themelio.

```
USAGE:
  themelio-vanity [PATTERN]

PATTERN:
  The pattern which the wallet shall start with, excluding the 
  mandatory 't' prefix. Allowed characters are:

  0123456789abcdefghjkmnpqrstvwxyz. Unavailable are i,l,o and u.

  Characters `i,l,o` will be replaced with 1 or 0 respectively.
  Character `u` has no "leet" substitute and is invalid.
```

(_Themelio uses base32 encoding with a restricted character set for better readability_)

## Building
Provided you have the Rust toolchain installed, call
`cargo build` from the project root or `rustc main.rs` from the `src` directory. There are also VS Code tasks for this.

## Support
Say thanks by sending me some $MEL:
`t9hhpfntbjcz6jy3882kb2a1c2ypfznz697hq2f6bwbsm2900yr9eg`