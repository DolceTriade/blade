# BLADE (Build Log Analysis Dashboard Engine)

Blade is a Bazel BEP viewer. It's in its early stages, but should be generally functional.

![screenshot](img/ss.png)

# Building

## Environment

Blade depends on bazel and nix to build. Nix is used to manage third party dependencies and bazel is used as the build system.

First, install Nix:
```
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Then run `nix develop --impure` in the source dir to set up the dev environment. Alternatiely, you can use direnv to automatically load the dev dependencies into the environment: `direnv allow`

## Running

In one terminal, run:
`bazel run //blade --db_path sqlite:///tmp/blade.db --allow_local`

Then, in another, run:

`bazel test -c opt --bes_backend=grpc://localhost:50332 --bes_results_url="http://localhost:3000/invocation/" //...`

to test it out.