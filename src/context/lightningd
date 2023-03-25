# syntax=docker/dockerfile:2
FROM  elementsproject/lightningd:latest

EXPOSE 9735 11001

ENV LIGHTNINGD_CHAIN=btc
ENV LIGHTNINGD_NETWORK=regtest
ENV LIGHTNINGD_RPC_PORT=18443

RUN mkdir shared 

ENTRYPOINT  ( \ 
        sleep 15s && \
        chmod -R 777 /root/.lightning/regtest/*  \
        ) & \
        ./entrypoint.sh \
        --network=regtest \
        --bitcoin-rpcuser=foo \
        --bitcoin-rpcpassword=qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0= \
        --bitcoin-rpcport=18443 \
        --log-level=debug \
        --bitcoin-rpcconnect=bitcoind \
        --grpc-port=11001
