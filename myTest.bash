privkey="1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8"
pubkey="f30544d6009c8d8d94f5d030b2e844b1a3ca036255161c479db1cca5b374dd1c"
script_alice="'[144 OP_CHECKSEQUENCEVERIFY OP_DROP 9997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803be OP_CHECKSIG]'"
script_bob="'[OP_SHA256 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG]'" 
vout=0 # CONFIRM THIS OR THINGS WILL FAIL
txin="020000000001010aa633878f200c80fc8ec88f13f746e5870be7373ad5d78d22e14a402d6c6fc20000000000feffffff02a086010000000000225120a5ba0871796eb49fb4caa6bf78e675b9455e2d66e751676420f8381d5dda8951c759f405000000001600147bf84e78c81b9fed7a47b9251d95b13d6ebac14102473044022017de23798d7a01946744421fbb79a48556da809a9ffdb729f6e5983051480991022052460a5082749422804ad2a25e6f8335d5cf31f69799cece4a1ccc0256d5010701210257e0052b0ec6736ee13392940b7932571ce91659f71e899210b8daaf6f17027500000000"
tx="020000000171f2f89c07c3b58c7b0cf3654ba049d28bbcc76b7298f41c17e7b1a3149040ec0000000000ffffffff01905f010000000000160014ceb2d28afdcad1ae0fc2cf81cb929ba29e83468200000000"
echo tap $pubkey 2 $script_alice $script_bob
echo 
echo tap --tx=$tx --txin=$txin $pubkey 2 $script_alice $script_bob 
echo 
echo tap --privkey=$privkey --tx=$tx --txin=$txin $pubkey 2 $script_alice $script_bob
echo
echo tap --tx=$tx --txin=$txin $pubkey 2 $script_alice $script_bob 1
# tap $pubkey 2 $script_alice $script_bob

MACAROON_HEADER="Grpc-Metadata-macaroon: $(xxd -ps -u -c 1000 /home/ash/.polar/networks/1/volumes/lnd/alice/data/chain/bitcoin/regtest/admin.macaroon)"
curl -X GET --cacert /home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert --header "$MACAROON_HEADER" https://localhost:8080/v1/fees 

curl -X GET --cacert /home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert  https://localhost:8080/v1/fees 