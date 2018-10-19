# `cargo template-ci` - automatically generate a fast travis.yml

In all my rust projects, I use travis to provide continuous
integration services, and to validate that things are
correct. However, rust's commandline utilities have changed a lot (and
clippy still does!), so I had to adjust the invocations in every
single `.travis.yml` I had. Not fun!

So here's a little tool that will generate a .travis.yml file for your
project according to a few flags, with a bunch of nice things:

* It uses a build matrix so that your builds are all fast.
* Lets you pick and choose which build types besides tests you want to
  run: By default, it runs clippy and checks rustfmt compliance. You
  can also activate benchmarks.
* Allows customizing the OS to run on and the versions to test with.

## Running

Once installed, you can run this tool in a rust project with `cargo
template-ci`. It will overwrite the project's existing .travis.yml, so
make sure you check if it did a good job!

## Configuring

By default, the configuration is as follows:

* Run tests on `stable`, `beta`, `nightly`.
* Run rustfmt on `stable`.
* Run clippy on `nightly`, but allow failures.
* Do not run benchmarks (but run them on `nightly` if enabled).

You can configure the generated config file by editing your project's
package metadata in `Cargo.toml`: Everything lives under the key
`package.metadata.template_ci`. This project has [an example that
makes clippy failures
fatal](https://github.com/antifuchs/cargo-template-ci/blob/a8740c68351cd99376c39b5906fde06e271e5e01/Cargo.toml#L27-L28).

Here's a list of configurable keys:

* `package.metadata.template_ci.os`: The operating system to run on. Defaults to `linux`
* `package.metadata.template_ci.dist`: The operating system distribution version to run on. Defaults to `xenial` (Ubuntu 16.04)
* `package.metadata.template_ci.versions`: The versions of rust to run tests on, in a build matrix. Defaults to `["stable", "beta", "nightly"]`

There are additional matrix build settings:

* `package.metadata.template_ci.rustfmt`: Settings for running an additional matrix build for checking rustfmt validity. Settings:
  * `run`: whether to run the build at all. Default: `true`.
  * `version`: what version to run on. Default: `stable`.
  * `allow_failure`: whether a non-zero exit status should break the build. Default `false`.

* `package.metadata.template_ci.clippy`: An additional matrix build for the `clippy` linter.
  * `run`: `true`
  * `version`: `nightly`
  * `allow_failure`: `true`

* `package.metadata.template_ci.bench`: An additional matrix build for running `cargo bench`.
  * `run`: `false`
  * `version`: `nightly`
  * `allow_failure`: `false`
