# BLADE (Build Log Analysis Dashboard Engine)

Blade is a Bazel BEP viewer. It's in its early stages, but should be generally functional.

![screenshot](img/ss.png)

# Quick Start

## Docker + Emphemeral SQLite

You can run this using Docker:

```
docker run --rm -p 3000:3000 -p 50332:50332 -v $HOME:$HOME ghcr.io/dolcetriade/blade:0.1 --db_path sqlite:///tmp/blade.db -l
```

Then you can run:

```
bazel test -c opt --bes_backend=grpc://localhost:50332 --bes_results_url="http://localhost:3000/invocation/" //...
```

If you want the database to be persistent, you bind mount a location in. You need to bind mount `$HOME` so you can access build logs. If you have a local remote cache, you can use that instead. You can override bytestream locations with alternate locations by passing in `-o bytestream://original.com=http://127.0.0.1:3834`. This flag can be passed in multiple times. It might be useful to set the bazel flag `--remote_bytestream_uri_prefix`. If you use a remote cache, you can omit the `-l` flag which prevents reading local files, which is a security risk since this would allow people to read arbitrary files on the host.

## PostgreSQL

There isn't a fully fleshed out example for this... Assuming you can access postgresql from the docker container:

```
docker run --rm -p 3000:3000 -p 50332:50332 ghcr.io/dolcetriade/blade:0.1 --db_path postgres://username:password@localhost/diesel_demo
```

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