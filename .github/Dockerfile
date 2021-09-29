FROM denoland/deno:ubuntu

ENV DEBIAN_FRONTEND=noninteractive

ENV LANG=en_US.UTF-8

RUN apt update

RUN apt install -y locales && locale-gen --purge en_US.UTF-8

RUN apt install -y curl jq tmux vim git

WORKDIR /data

COPY ./bin/nftmart-node /usr/bin/nftmart-node

RUN chmod +x /usr/bin/nftmart-node

COPY ./bin/nftmart-aura-node /usr/bin/nftmart-aura-node

RUN chmod +x /usr/bin/nftmart-aura-node

COPY scripts/rotate-key /usr/bin/rotate-key

RUN chmod +x /usr/bin/rotate-key

EXPOSE 30333

RUN ln -sf nftmart-node /usr/bin/nftmart

ENTRYPOINT ["nftmart-node"]
