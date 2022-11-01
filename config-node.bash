#!/bin/bash
# echo hello

user=foo
password=qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0=
url=http://127.0.0.1:18443/
address='"addr(bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj)"';
descriptor='"addr(bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj)#3gv8dgag"';
# descriptor='"addr(bcrt1ppjj995khlhftanw7ak4zyzu3650rlmpfr9p4tafegw3u38h7vx4qnxemeg)#hzc3j2sf"'

function invoke {
	echo $1
	curl --user $user:$password --data-binary "${1}" -H 'content-type: text/plain;' $url  | python -mjson.tool
}

function getdescriptorinfo {
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method": "getdescriptorinfo", "params": '[$1]'}'
	invoke "${JSON_STRING}" 

}

function createwallet {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "createwallet", "params": ["my_wallet", true, false,"",false,true,true]}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method": "createwallet", "params": ["my_wallet", true, false,"",false,true,true]}'
	invoke "${JSON_STRING}" 
}

function importdescriptors {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "importdescriptors", "params": [[{"desc":"addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs","timestamp":"now"}]]}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "importdescriptors", "params": [[{"desc":'$1',"timestamp":"now"}]]}'
	invoke "${JSON_STRING}" 
}

function generatetodescriptor {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "generatetodescriptor", "params": [100, "addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs"]}' -H 'content-type: text/plain;' $url  | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "generatetodescriptor", "params": ['$1', '$2']}'
	invoke "${JSON_STRING}" 
}
function listunspent {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "listunspent", "params": []}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "listunspent", "params": []}'
	invoke "${JSON_STRING}" 
}

case $1 in

  all)
    echo -n "running complete script"
		getdescriptorinfo $address
		createwallet
		importdescriptors $descriptor
		generatetodescriptor 50 $descriptor 
		listunspent
    ;;

  addr)
	importdescriptors $descriptor
	generatetodescriptor 50 $descriptor 
	listunspent
  ;;

  descinfo) 
	getdescriptorinfo $address
	;;
	
  import) 
	importdescriptors $descriptor 
	;;

  mine)
	generatetodescriptor $2 $descriptor  
	;;

  unspent)
	listunspent
    ;;
  

  init)
	docker run --rm -it   -p 18443:18443   -p 18444:18444   ruimarinho/bitcoin-core   -printtoconsole   -regtest=1   -rpcallowip=172.17.0.0/16   -rpcbind=0.0.0.0   -rpcauth='foo:7d9ba5ae63c3d4dc30583ff4fe65a67e$9e3634e81c11659e3de036d0bf88f89cd169c1039e6e09607562d54765c649cc'
	;;
  *)
    echo -n "unknown"
    ;;
esac


# function all {
# 	
# }
