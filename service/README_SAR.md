# iam-ssh-agent

A replacement ssh-agent that uses the caller's IAM identity to access a list of
permitted ssh identities.

`iam-ssh-agent` is designed to be used in less trusted continuous integration
environments where you want to use an ssh key to clone source control
repositories without granting access to the raw key material.

`iam-ssh-agent` is split into two components: a binary that binds a unix domain
socket with the ssh-agent protocol, and a serverless API that uses API Gateway
and Lambda functions to answer `list keys` and `sign data` requests.

See the [GitHub repository](https://github.com/buildkite/iam-ssh-agent) for
more documentation.