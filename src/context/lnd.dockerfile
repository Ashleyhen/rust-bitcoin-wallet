# syntax=docker/dockerfile:2
FROM lightninglabs/lnd:v0.16.4-beta.rc1
EXPOSE 9730 8080 10006

RUN mkdir shared 
RUN cd  && \
   echo 'alias lncli="lncli --network regtest --rpcserver localhost:10006"' >> .bashrc && \
   source .bashrc

ENTRYPOINT ( \ 
        sleep 10 && \
        cp -r /root/.lnd/data/chain/bitcoin/regtest/* /shared/ && \
        cp /root/.lnd/tls.* /shared/ && \
        chmod -R 777 /shared/ \ 
     ) & \
        lnd \
        --bitcoin.active \
        --accept-amp \
        --debuglevel=debug \
        --bitcoin.node=bitcoind \
        --bitcoin.regtest \
        --bitcoind.rpchost=bitcoind:18443 \ 
        --bitcoind.rpcpass=qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0= \
        --bitcoind.rpcuser=foo \
        --bitcoind.zmqpubrawblock=tcp://bitcoind:28334 \
        --bitcoind.zmqpubrawtx=tcp://bitcoind:28335 \
        --db.postgres.timeout=0 \
        --listen=0.0.0.0:9730 \
        --noseedbackup \
        --restlisten=0.0.0.0:8080 \
        --rpclisten=0.0.0.0:10006 \
        --trickledelay=5000 \
        --minchansize=0 \
        --tlsextradomain=lnd