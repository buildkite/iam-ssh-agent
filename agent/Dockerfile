FROM debian:buster-slim

RUN apt-get update && \
    apt-get install -y \
      busybox \
      ca-certificates \
      curl && \
    rm -rf /var/lib/apt/lists/*

ARG IAM_SSH_AGENT_VERSION
RUN cd /tmp && \
    curl --location "https://github.com/keithduncan/iam-ssh-agent/releases/download/v${IAM_SSH_AGENT_VERSION}/iam-ssh-agent_${IAM_SSH_AGENT_VERSION}_amd64.deb" >iam.deb && \
    dpkg -i iam.deb && \
    rm iam.deb
