# CogRS

[![Rust](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml/badge.svg)](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml) [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Toy project to learn Rust and also make a simple CLI that behaves similarly to ansible CLI.

## Examples

```bash
$ cogrs ../ansible-playground/inventory --list-hosts 'azure, k8s' --limit 'mysql_a,mysql_b,control?.local*[0]'
```