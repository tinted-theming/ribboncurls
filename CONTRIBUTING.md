# Contributing to Ribboncurls

First of all, thank you for considering to contribute to [Ribboncurls]!

## Code of Conduct

This project and everyone participating in it is governed by the
Ribboncurls Code of Conduct. By participating, you are expected to
uphold this code.

## Getting Started

- Clone Ribboncurls with git
- Install Rust, `cargo-about` and `cargo-deny` or run `make install`
  from the cloned repo root.
- Build: `make build`
- Test: `make test`

## Reporting Issues

Before creating bug reports, please check a [list of known issues] to
see if the problem has already been reported. If you're unable to find
an open issue addressing the problem, open a new one. Be sure to include
a title and clear description, as much relevant information as possible,
and a code sample or an executable test case demonstrating the expected
behavior that is not occurring.

## Pull Request Process

1. Update the workspace README.md with details of changes, this includes
   new environment variables, exposed ports, useful file locations, and
   container parameters.
1. PR should be passing the CI tests. Make sure to run `cargo fmt` and
   `cargo check` before creating a Pull Request.
1. Include either doc tests or integration tests for your changes if
   relevant.
1. Update the version to a new relevant version. The versioning scheme
   we use is [SemVer].
1. Ensure any install or build dependencies are removed before the end
   of the layer when doing a build.
1. Once we are happy with the Pull Request we will merge the changes.

Thank you for your contribution!

[list of known issues]: https://github.com/evestera/ribboncurls/issues
[SemVer]: http://semver.org/
