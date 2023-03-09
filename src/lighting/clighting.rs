pub fn sendto_addr(){
// script_demo();
    // Client::new(sockpath)
    let client=clightningrpc::LightningRPC::new("./../../.docker/volumes/lightningd_data/lightning-rpc");
    let client2=clightningrpc::LightningRPC::new("./../../.docker/volumes/lightningd2_data/lightning-rpc");


    dbg!(client2.getinfo().unwrap());
    let id =client2.getinfo().unwrap().id;
    let result=client.connect(&id, Some("10.5.0.5:19846")).unwrap();
    // client.invoice(msatoshi, label, description, expiry)
    // client.invoice(msatoshi, label, description, expiry)
    dbg!(result);
    let fund_addr=client.newaddr(None).unwrap();
    dbg!(fund_addr);
}