# FROM ubuntu:20.10
FROM debian:buster

RUN apt update && apt install -y curl

WORKDIR /data

ADD substrate /usr/bin/nftmart
ADD staging_spec_raw.json /data

RUN chmod +x /usr/bin/nftmart

ENTRYPOINT ["nftmart"]
