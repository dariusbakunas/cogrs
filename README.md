# CogRS

[![Rust](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml/badge.svg)](https://github.com/dariusbakunas/cogrs/actions/workflows/rust.yml) [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0) [![Coverage Status](https://coveralls.io/repos/github/dariusbakunas/cogrs/badge.svg?branch=main)](https://coveralls.io/github/dariusbakunas/cogrs?branch=main)

CogRS is a Rust-based tool for automating tasks on remote machines, inspired by Ansible but designed to work seamlessly in more restrictive environments. Unlike Ansible, CogRS does not require Python to be installed on the remote machines, making it lightweight and easier to deploy in various environments.

This project serves two primary purposes:
- **Reduce remote machine dependencies** by leveraging the performance and portability of Rust.
- **Provide a learning opportunity** for Rust and Ansible, combining lessons from both ecosystems.

The tool supports using Ansible-like inventories and playbooks, allowing users to leverage familiar workflows while introducing the flexibility of Rust.

---

## Features

- **Python-Free Remote Operations**: Execute commands on remote hosts without requiring Python or other interpreters on the target machines.
- **Ansible Compatibility**: Partial support for Ansible inventories, playbooks and vaults for easier adoption.
- **Host Filtering**: Use flexible host matching with patterns, groups, and limit options to target specific machines.
- **CLI Simplicity**: A single binary with a familiar and intuitive CLI experience.

---

## Why CogRS?

While Ansible is a powerful provisioning tool, its reliance on Python can limit its usability in environments where installing dependencies is restricted. CogRS offers a solution by eliminating the need for Python or any additional software on remote machines. This makes it ideal for environments like:
- Restricted enterprise setups
- Minimal or embedded operating systems
- Custom-built or containerized environments

In return, CogRS provides a lightweight alternative for users already familiar with Ansible-style workflows, minimizing the learning curve.

---

## Installation

⚠️ Installation instructions coming soon. This is currently a work-in-progress project.

---

## Usage

Below is an example of how to list specific hosts from an inventory using CogRS:

```bash
$ cogrs ../ansible-playground/inventory --list-hosts 'azure,k8s' --limit 'mysql_a,mysql_b,control?.local*[0]'
```

- `--list-hosts`: Lists hosts from the inventory based on the specified patterns.
- `--limit`: Applies additional filters to narrow down the selected hosts.

More documentation and detailed usage examples will be added as the project evolves.

---

## Roadmap

Features planned for future releases include:
- Expanded compatibility with Ansible inventories and playbooks (partial, no plugin/module support).
- Support for custom task execution (similar to Ansible modules).
- Enhanced logging and debugging options.
- Expanding supported connection types (SSH, etc.).

---

## Contributing

This is currently a learning project, so feel free to submit issues, feature requests to help improve CogRS.

For ideas or discussions, start a thread in the [GitHub Issues](https://github.com/dariusbakunas/cogrs/issues) section.

---

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).