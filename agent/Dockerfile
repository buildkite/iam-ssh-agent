FROM debian:buster

RUN apt-get update && \
	apt-get install -y \
	ca-certificates

COPY target/debian /packages
RUN dpkg -i /packages/*
