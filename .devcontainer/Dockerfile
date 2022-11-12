FROM ruimarinho/bitcoin-core:latest
RUN apt update
RUN apt install git -y
RUN apt install cargo -y
RUN apt install python3 -y
RUN apt install jq
RUN alias bitcoind='bitcoind -chain=regtest'
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

