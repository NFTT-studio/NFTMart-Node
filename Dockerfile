FROM btwiuse/substrate-builder:nightly-2021-09-01 as builder

ADD . /build

WORKDIR . /build

RUN cargo build --release

FROM denoland/deno:ubuntu

ENV DEBIAN_FRONTEND=noninteractive

ENV LANG=en_US.UTF-8

RUN apt update

RUN apt install -y locales && locale-gen --purge en_US.UTF-8

RUN apt install -y curl jq tmux vim git

# RUN curl -sL https://deb.nodesource.com/setup_16.x | bash - && apt install -y nodejs

# RUN npm install -g subgenius

WORKDIR /data

ADD --from=builder /build/target/release/nftmart-node /usr/bin/nftmart-node

RUN chmod +x /usr/bin/nftmart-node

ADD --from=builder /build/target/release/nftmart-aura-node /usr/bin/nftmart-aura-node

RUN chmod +x /usr/bin/nftmart-aura-node

ADD scripts/rotate-key /usr/bin/rotate-key

RUN chmod +x /usr/bin/rotate-key

EXPOSE 30333

RUN ln -sf nftmart-node /usr/bin/nftmart

ENTRYPOINT ["nftmart-node"]
