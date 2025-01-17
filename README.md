# CogRS

[![Rust](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml/badge.svg)](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml)

Toy project to learn Rust and also make a simple CLI that behaves similarly to ansible CLI.

## Examples

```bash
$ cogrs -m "shell" -a "ls -al" -i ./inventory.yml workers
```