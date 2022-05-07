# toolchain-bootstrap

This project provides functionality for bootstrapping / building compiler
toolchains with high determinism.

**This project is in an alpha state and the toolchains built with it are
not as portable or useful as we would like. Please do not use as generic
toolchains until this disclaimer is removed.**

# How It Works

A container image of Debian Jessie is constructed using snapshot.debian.org
to provide a deterministic snapshot of the package repository.

This container is used to build modern versions of binutils and GCC,
which is necessary since the version of GCC/Clang in Debian Jessie cannot
build modern GCC/LLVM/Clang.

# Usage

We use [Earthly](https://earthly.dev/) for coordinating build activity
across multiple containers.

Once you install Earthly, run `earthly +<target>` to build a target.

You will need to pass some environment variables / secrets into Earthly
so sccache can be used. To do so, create an ``.env`` file in this directory.
Copy the ``.env.example`` file for a template.
