# iam-ssh-agent

A replacement ssh-agent that uses the caller's IAM identity to access a list of
permitted ssh identities.

iam-ssh-agent is split into two components; a serverless API that uses API
Gateway and Lambda functions to list keys and sign data, and a binary that binds
a unix domain socket with the ssh-agent protocol.

The [`/agent`](https://github.com/keithduncan/iam-ssh-agent/tree/master/agent)
subdirectory contains a Rust crate that builds the `iam-ssh-agent` binary.

The [`/service`](https://github.com/keithduncan/iam-ssh-agent/tree/master/service)
subdirectory contains an AWS SAM project that deploys the serverless backend
for the `iam-ssh-agent` binary.

## Agent

`iam-ssh-agent` is designed to be used in less trusted continuous integration
environments where you want to use an ssh key to clone source control
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
workload or a separate account. You can also choose whether to create a Regional
or Private endpoint. Access to a Regional endpoint can be restricted by
AWS Account ID, while a Private endpoint allows fine grained restriction by
source VPC or VPC Endpoint.

Keys are stored in AWS Systems Manager Parameter Store where the private keys
can be encrypted with a KMS key.

Key permissions are stored in a DynamoDB table keyed by IAM Entity Unique ID.

Deploy the service to an account using the `sam` command line tool or AWS
Serverless Application Repository [TBD].

Once you have successfully deployed the service you can [add keys](#adding-keys),
[grant access](#granting-access-to-keys), and [test access](#testing-access).

### Adding Keys

You can add keys to AWS Systems Manager Parameter Store using the AWS CLI or
Console. The public and private keys are stored in separate parameters, the
ListIdentities lambda is granted access to the public key parameters and the
GetSignature lambda is granted access to both.

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

Use the AWS CLI look up the Unique ID for the IAM entity to be granted access.
These are not exposed in the AWS Console, for roles use `get-role` and copy
the `RoleId`:

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
create an item in the DynamoDB permissions table. You can add an item using the
AWS CLI or Console.

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
by whether you deploy the service to the same account as your calling IAM entities
or a different account.

When using an API Gateway deployed to the same account, you don't have to provide
explicit access in the IAM entity policies, the API Gateway Resource Policy is
enough to grant access. An explicit deny in an IAM Policy is of course
respected.

When using an API Gateway deployed to a separate account (cross account IAM
access) you must configure access in both accounts. The API Gateway's resource
policy must include the calling account’s account ID (or source VPC / VPC Endpoint
if using a Private endpoint), and the calling account’s IAM entity must be granted
an explicit allow with an IAM Policy Statement:

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

### Private Endpoint Configuration

When configuring the API Gateway with `EndpointConfiguration: PRIVATE` some
additional configuration may be necessary.

First you must configure a VPC Endpoint for `execute-api` in the VPC you want
to have access to the API Gateway. Ensure the VPC Endpoint security groups
allow inbound network traffic from the security group that `iam-ssh-agent`
binary will execute in.

If Private DNS is enabled, all `execute-api` requests from this VPC will be
routed via this endpoint. If this is not appropriate for your VPC, you can
[associate the API Gateway with the VPC Endpoint](https://docs.aws.amazon.com/apigateway/latest/developerguide/associate-private-api-with-vpc-endpoint.html)
and use the Route 53 alias DNS name for your private API gateway instead.

`iam-ssh-agent` supports connecting to Private DNS Names or a Route 53 alias, it
does not support Endpoint-Specific Public DNS Hostnames. See
[How to Invoke a Private API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-private-api-test-invoke-url.html) for details on these options.

You may wish to change the API Gateway Resource Policy (ensure you redeploy the
API after any changes) or set a [VPC Endpoint Policy](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-vpc-endpoint-policies.html)
to restrict access from inside your VPC to the `iam-ssh-agent` backend.

See the [AWS Private API troubleshooting guide](https://aws.amazon.com/premiumsupport/knowledge-center/api-gateway-private-endpoint-connection/)
for more tips on troubleshooting access to Private API Gateways.

## Testing Access

I use this project to provide my [Buildkite](https://buildkite.com) agents
[running on ECS](https://github.com/keithduncan/buildkite-on-demand) access to
ssh keys to clone source code repositories.

To use the `iam-ssh-agent` service in ECS Tasks I add a sidecar container which
uses the [keithduncan/iam-ssh-agent](https://hub.docker.com/repository/docker/keithduncan/iam-ssh-agent)
docker image to my task definitions. The task definition also uses a bind mount
volume to expose the unix domain socket bound by `iam-ssh-agent` to the Buildkite
agent container which invokes `ssh`.

To ensure the `iam-ssh-agent` container has booted, the main container uses a
container dependency `DependsOn: [{"Condition": "HEALTHY", "ContainerName": "ssh-agent"}]`
condition to wait for the ssh-agent to boot and become healthy before starting,
and the sidecar container defines a healthcheck which uses busybox to verify the
socket has been bound.

An example task definition which prints which keys the container has access
to, based on the task IAM role passed when scheduling the ECS task, is shown
below. This task definition can be useful for diagnosing `ssh` access issues
and confirming that your ECS Task Role has access to the keys you expect.

```yaml
SshTaskDefinition:
  Type: AWS::ECS::TaskDefinition
  Properties:
    Family: ssh-example
    ContainerDefinitions:
      - Name: agent
        EntryPoint:
          - /bin/sh
          - -c
        Command:
          - ssh-add -L; ssh -vvvvT git@github.com
        Essential: true
        Image: buildkite/agent:3
        LogConfiguration:
          LogDriver: awslogs
          Options:
            awslogs-region: !Ref AWS::Region
            awslogs-group: /aws/ecs/ssh
            awslogs-stream-prefix: ecs
        Environment:
          - Name: SSH_AUTH_SOCK
            Value: /ssh/socket
        DependsOn:
          - Condition: HEALTHY
            ContainerName: ssh-agent
        MountPoints:
          - ContainerPath: /ssh
            SourceVolume: ssh-agent
      - Name: ssh-agent
        Command:
          - /usr/bin/iam-ssh-agent
          - daemon
          - --bind-to=/ssh/socket
        Essential: true
        Image: keithduncan/iam-ssh-agent:latest
        Environment:
          - Name: IAM_SSH_AGENT_BACKEND_URL
            Value: !Ref YourIamSshAgentBackendUrlHere
        LogConfiguration:
          LogDriver: awslogs
          Options:
            awslogs-region: !Ref AWS::Region
            awslogs-group: /aws/ecs/ssh
            awslogs-stream-prefix: ecs
        HealthCheck:
          Command:
            - /bin/busybox
            - test
            - -S
            - /ssh/socket
        MountPoints:
          - ContainerPath: /ssh
            SourceVolume: ssh-agent
    Cpu: 256
    Memory: 512
    NetworkMode: awsvpc
    ExecutionRoleArn: !ImportValue agent-scheduler-ECSTaskExecutionRoleArn
    RequiresCompatibilities:
      - FARGATE
    Volumes:
      - Name: ssh-agent
```

Since my `iam-ssh-agent` service is deployed to a separate AWS account, my
ECS task role has a policy to explicitly grant access to the API Gateway:

```yaml
ProjectRole:
  Type: AWS::IAM::Role
  Properties:
    Path: /BuildkiteAgentTask/
    RoleName: ProjectName
    AssumeRolePolicyDocument:
      Statement:
      - Effect: Allow
        Principal:
          Service: [ecs-tasks.amazonaws.com]
        Action: ['sts:AssumeRole']
    Policies:
      - PolicyName: SshAgentApi
        PolicyDocument:
          Statement:
            - Effect: Allow
              Action: execute-api:Invoke
              Resource:
                !Ref YourIamSshAgentApiGatewayArnHere
```

For more details on running Buildkite agents on-demand with ECS see my
[agent-scheduler](https://github.com/keithduncan/buildkite-on-demand/tree/master/agent-scheduler)
project.
