# iam-ssh-agent

An ssh-agent replacement that uses IAM credentials and identity to grant access
to ssh identities.

iam-ssh-agent is split into two components; a serverless API that uses API
Gateway and Lambda functions to list and sign data, and a binary that binds a
unix domain socket and speaks the ssh-agent protocol. The agent should be a drop
in replacement for existing uses of ssh-agent.

iam-ssh-agent is designed to be used in less trusted environments like
continuous integration where you want to delegate use of an SSH key without
granting access to the raw key material.

Key permissions are stored in a DynamoDB table keyed by IAM Entity Unique ID
and the keys themselves are stored in AWS Systems Manager where the private keys
can be encrypted.