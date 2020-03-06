# iam-ssh-agent agent

The `agent` project is a Rust crate that builds a binary.

The artifacts built from this project are:

- `iam-ssh-agent.deb` a debian package, attached to the GitHub release
- `keithduncan/iam-ssh-agent:latest` a Docker image, pushed to Docker hub
	- This is currently Debian based but I want to replace it with an Alpine image