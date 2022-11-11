FROM ruimarinho/bitcoin-core:latest
RUN apt update
RUN apt install git -y
RUN apt install cargo -y
RUN alias bitcoind='bitcoind -chain=regtest'
