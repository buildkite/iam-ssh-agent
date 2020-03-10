# iam-ssh-agent agent

Binary replacement for `ssh-agent` that uses an [iam-ssh-agent](https://github.com/keithduncan/iam-ssh-agent)
backend for `list keys` and `sign data` operations.

The artifacts built from this crate are:

- `iam-ssh-agent.deb` a debian package, attached to the GitHub release
- `keithduncan/iam-ssh-agent:latest` a Docker image, pushed to Docker hub (currently based on Debian but I want to replace it with an Alpine base)