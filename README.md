# COSMIC Application Template

A template for developing applications for the COSMIC™ desktop environment.

## Getting Started

Log into your GitHub account and click the "Use this template" button above. This will create a new repository in your account. Choose a name for this repository, and then clone it locally onto your system. Make the following changes after cloning it:

- In `Cargo.toml`, change the `name` and set your `license` and `repository`.
- Create a `LICENSE` file containing your chosen software license.
- Rename the `cosmic_app_template` portion of `i18n/en/cosmic_app_template.ftl` to the new crate `name`.
- In `justfile`, change the `name` and `appid` variables with your own.
- In `src/app.rs`, change the `APP_ID` value in the `Application` implementation of the `AppModel`.
- In `src/app.rs`, change the `REPOSITORY` const with the URL to your application's git repository.
- In `res/app.desktop`, change the `Name=`, `Exec=`, and `Icon=` fields
- Set your license within the SPDX tags at the top of each source file

A [justfile](./justfile) is included by default with common recipes used by other COSMIC projects. Install from [casey/just][just]

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Documentation

Refer to the [libcosmic API documentation][api-docs] and [book][book] for help with building applications with [libcosmic][libcosmic].

[api-docs]: https://pop-os.github.io/libcosmic/cosmic/
[book]: https://pop-os.github.io/libcosmic-book/
[libcosmic]: https://github.com/pop-os/libcosmic/
[just]: https://github.com/casey/just