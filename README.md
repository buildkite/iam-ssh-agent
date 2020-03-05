# iam-ssh-agent

A replacement ssh-agent that uses the caller's IAM identity to access a list of
permitted ssh identities.

iam-ssh-agent is split into two components; a serverless API that uses API
Gateway and Lambda functions to list and sign data, and a binary that binds a
unix domain socket with the ssh-agent protocol.

## Agent

iam-ssh-agent is designed to be used in less trusted environments like
continuous integration where you want to use an ssh key to clone source control
repositories without granting access to the raw key material.

The agent binary should be a near drop in replacement for existing uses of
ssh-agent, or provide a pathway for you to remove private key material from
continuous integration hosts.

The agent requires an `IAM_SSH_AGENT_BACKEND_URL` environment variable to be set
to the URL of the API Gateway stage it should connect to. This URL will be
printed following a successful deploy of the service and look like
`https://{api-gateway-id}.execute-api.{region}.amazonaws.com/Prod`.

The agent binary will auto discover IAM credentials in the expected places;
environment variables, EC2 instance metadata or ECS task metadata. Requests to
the API Gateway will be signed with these credentials and the service will
provide access to keys listed in the DynamoDB Permissions table for the caller's
IAM entity.

## Service

You can choose whether to deploy the service to the same account as your CI
workload or a separate account. Access to the API Gateway is restricted by AWS
Account ID. If you wanted to further restrict access you might consider creating
a Private API Gateway and using VPC endpoints.

Keys are stored in AWS Systems Manager Parameter Store where the private keys
can be encrypted with a KMS key.

Key permissions are stored in a DynamoDB table keyed by IAM Entity Unique ID.

Deploy the service to an account using the `sam` command line tool or AWS
Serverless Application Repository [TBD].

Once you have successfully deployed the service you can [add keys](#adding-keys)
and [grant access](#granting-access-to-keys).

### Adding Keys

You can add keys using the AWS Console. Keys are stored in the AWS Systems
Manager Parameter Store. The public and private keys are stored separately. The
ListIdentities lambda is granted access to the public keys and the GetSignature
lambda is granted access to both.

You can use any hierarchy to store your public and private keys in SSM so long
as the parameter paths end in `key.pub` and `key` respectively.

The GetSignature lambda IAM role includes a policy that permits `kms:Decrypt`
using the `aws/ssm` KMS key. You can store your ssh private keys in a
`SecureString` parameter encrypted with that key to prevent unintended access to
the raw key material. Deploying this service to a unique AWS account can also
help limit access to the key material.

Example key hierarchies:

```
# GitHub repository deploy key
/github/keithduncan/iam-ssh-agent/key.pub
/github/keithduncan/iam-ssh-agent/key

# Machine user key
/github/machine-user1/key.pub
/github/machine-user1/key

# GitLab Global Key
/gitlab.company.com/global/name/key.pub
/gitlab.company.com/global/name/key
```

### Granting Access to Keys

Once you have [added the keys](#adding-keys) to the parameter store, you can
grant access to those keys to IAM entities.

Use the AWS CLI look up the Unique ID for the IAM entity, these are not exposed
in the AWS Console, for roles use `get-role` and copy the `RoleId`:

```
aws iam get-role --role-name MyRole
{
    "Role": {
        "Path": "/",
        "RoleName": "MyRole",
        "RoleId": "AROAXXXXXXXXXXXXXXXXX",
        "Arn": "arn:aws:iam::{account}:role/MyRole",
        "CreateDate": "2020-01-22T10:52:29Z",
        "AssumeRolePolicyDocument": {
            "Version": "2008-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": {
                        "Service": "ecs-tasks.amazonaws.com"
                    },
                    "Action": "sts:AssumeRole"
                },
                {
                    "Effect": "Allow",
                    "Principal": {
                        "AWS": "arn:aws:iam::{account}:root"
                    },
                    "Action": "sts:AssumeRole"
                }
            ]
        },
        "Description": "",
        "MaxSessionDuration": 3600,
        "RoleLastUsed": {
            "LastUsedDate": "2020-03-04T04:38:00Z",
            "Region": "us-east-1"
        }
    }
}
```

See [IAM Identifiers Unique ID](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_identifiers.html#identifiers-unique-ids)
for more details on IAM Unique IDs.

Once you have the Unique ID for the entity you want to grant access to, you can
create an item in the DynamoDB table. You can add an item using the AWS CLI or
console.

```
aws dynamodb update-item \
--table-name MyPermissionsTableName \
--key '{"IamEntityUniqueId":{"S":"AROAXXXXXXXXXXXXXXXXX"}}' \
--update-expression 'ADD #p :keys' \
--expression-attribute-names '{"#p":"Parameters"}' \
--expression-attribute-values '{":keys":{"SS":["/github/machine-user1/key"]}}'
```

#### Multiple Keys

An ssh-agent with multiple keys can cause issues when authenticating to source
control services. If an IAM entity with access to multiple keys connects to
`github.com` the ssh server will accept the first ssh key offered and
_authenticate_ successfully but later fail _authorization_ when attempting to
clone a repository that the key doesn't have access to.

Instead of using multiple keys, consider using a GitHub machine user or a GitLab
Global Deploy key whose access is limited to specific repositories.

### Granting Access to the API Gateway

How to grant access to the API Gateway to list keys and sign data is determined
by whether you deploy the service to the same account as your calling IAM roles
or a different account.

When using an API Gateway deployed to the same account access you don't have to
provide explicit access in the IAM entity policies, the API Gateway resource is
enough to grant access. An explicit deny on an IAM entity is of course
respected.

When using an API Gateway deployed to a separate account (cross account IAM
access) you must allow access in both accounts. The API Gateway's resource
policy must include the calling account’s account ID, and the calling account’s
IAM entity must be granted an explicit allow with an IAM Policy Statement:

```json
{
    "Statement": [
        {
            "Action": "execute-api:Invoke",
            "Resource": "arn:aws:execute-api:{region}:{account-id}:{api-gateway-id}/Prod/*/*",
            "Effect": "Allow"
        }
    ]
}
```

See the [API Gateway Authorization Flow](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-authorization-flow.html)
documentation for more details.
