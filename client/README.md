# client

Use the [ssh-agent](https://www.npmjs.com/package/ssh-agent) npm package as an
SSH Agent client for testing the IAM SSH Agent against the stock ssh-agent
implementation.

## Installation

```
npm install
```

## Prepare Agents

Run two agents side by side with these commands in separate sessions:

```
cd ../agent && aws-vault exec [aws profile] -- cargo run --quiet -- --bind-to=tmp/iam
ssh-agent -d -a ../agent/tmp/original
```

## List Keys

```
export SSH_AUTH_SOCK=path/to/sock
./node_modules/ssh-agent/bin/ssha-list
```

## Sign Data

```
export SSH_AUTH_SOCK=path/to/sock
./node_modules/ssh-agent/bin/ssha-sign [key name from list keys] < <(echo data)
```
