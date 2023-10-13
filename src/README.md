### 1. ROAD MAP

&checkmark; p2wpkh  
&checkmark; p2tr  
&checkmark; combine mutliple utxo's into a single utxo output  
&checkmark; create a easy to use framework  
&checkmark; script-spend  
&checkmark; multi-sig  
&checkmark; atomic swap  
[ ] access the lighting network 


1) start a bitcoin node
2) send some bitcoin to the addresses in the ``config-node.bash``
3) run the following: ``cargo run``




client API
```bash
curl -X POST -H "Content-Type: application/json" -d '{"bolt11": "lnbcrt200u1pjtkqtxpp5ker24hccwj77hgczkm27jyd7wkj6j803eyd4dfqhnxxcm04xslwqdq2wpshjgrdv5cqzzsxqyz5vqfppqzvsdwjay5x69088n27h0qgu0tm4u6gwqsp5mg4hgkewsns4d8j4ane5q750zv59upzhd0p4ljq66dgfx6p39cdq9qyyssqkzr9na5n98gmfw0wke7jxhw63c9yxe2gn29a2ey9ln7cpncryvj9wfycf2ewuayduzmex9rtt0ksj45yjazz08hdqdefwr7wppy7vhspsw2p6c", "id": 250}' http://localhost:8000/lnurl
 ```