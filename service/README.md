# iam-ssh-agent service

The `service` project is an AWS SAM template that deploys an API Gateway,
Lambdas, and a DynamoDB table.

## Deploying

The template can be deployed using the AWS Serverless Application Repository or
the [AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-reference.html#serverless-sam-cli).

You can choose whether to deploy the service to the same account as your CI
workload or to a separate account.

You can also choose whether to create a Regional or Private endpoint. A
Regional endpoint can be accessed from anywhere (including the public Internet),
while a Private endpoint can only be accessed from a VPC using a VPC Endpoint.
Access to a Regional endpoint can be restricted by AWS Account ID, while a
Private endpoint allows fine-grained restriction by source VPC or VPC Endpoint.

### Deploy from the AWS Serverless Application Repository web console

[![Deploy AWS Serverless Application](https://cdn.rawgit.com/buildkite/cloudformation-launch-stack-button-svg/master/launch-stack.svg)](https://serverlessrepo.aws.amazon.com/applications/arn:aws:serverlessrepo:us-east-1:832577133680:applications~iam-ssh-agent)

Open the application in AWS Console and click Deploy, fill in the parameters
as described below for a CLI deployment.

### Deploy using the Serverless Application Model on the command line

The AWS SAM CLI is an extension of the AWS CLI that adds functionality for
building and testing Lambda applications.

Before continuing, [install the AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html) if not already installed. The following
instructions were written using SAM Version 0.43.0.

To deploy `iam-ssh-agent` for the first time, authenticate with AWS and run the
following command from the `service` directory:

```bash
iam-ssh-agent$ sam deploy --guided
```

You will be prompted for the following parameter values:

* **Stack Name**: AWS will create a CloudFormation Stack using this name, use
something descriptive like `iam-ssh-agent`.
* **AWS Region**: The AWS region you want to deploy this instance of
`iam-ssh-agent`.
* **Parameter Endpoint Configuration**: Select REGIONAL or PRIVATE. 
* **Parameter AccountIds**: Limit access based on the account of the calling IAM
entities, this is recommended for REGIONAL endpoint configurations. You can
enter multiple AWS Account IDs, separated by commas.
* **Parameter SourceVpcIds**: Limit access based on the source VPC or VPC
Endpoint of the calling IAM entities, this is only available for PRIVATE
endpoint configurations.

Once the deployment is complete, `sam` will print the URL of the API Gateway and
the name of the DynamoDB permissions table. You can now add [keys](../README.md#adding-keys)
and [permissions](../README.md#granting-access-to-keys) to your agent backend.
