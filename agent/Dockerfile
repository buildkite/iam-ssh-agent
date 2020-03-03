FROM debian:buster-slim

RUN apt-get update && \
	apt-get install -y \
	ca-certificates && \
	rm -rf /var/lib/apt/lists/*

COPY target/debian /packages
RUN dpkg -i /packages/* && rm -r /packages
