# iam-ssh-agent agent

Binary replacement for `ssh-agent` that uses an [iam-ssh-agent](https://github.com/buildkite/iam-ssh-agent)
backend for `list keys` and `sign data` operations.

The artifacts built from this crate are:

- `iam-ssh-agent.deb` a Debian package, attached to the GitHub Release
- `buildkite/iam-ssh-agent:latest` a Docker image with Alpine base, pushed to Docker Hub