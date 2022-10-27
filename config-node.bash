#!/bin/bash
# echo hello

user=foo
password=qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0=
url=http://127.0.0.1:18443/
address='"addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)"';
descriptor='"addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs"';

function invoke {
	echo $1
	curl --user $user:$password --data-binary "${1}" -H 'content-type: text/plain;' $url  | python -mjson.tool
}

function getdescriptorinfo {
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method": "getdescriptorinfo", "params": '[$address]'}'
	invoke "${JSON_STRING}" 

}

function createwallet {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "createwallet", "params": ["my_wallet", true, false,"",false,true,true]}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method": "createwallet", "params": ["my_wallet", true, false,"",false,true,true]}'
	invoke "${JSON_STRING}" 
}

function importdescriptors {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "importdescriptors", "params": [[{"desc":"addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs","timestamp":"now"}]]}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "importdescriptors", "params": [[{"desc":'$descriptor',"timestamp":"now"}]]}'
	invoke "${JSON_STRING}" 
}

function generatetodescriptor {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "generatetodescriptor", "params": [100, "addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs"]}' -H 'content-type: text/plain;' $url  | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "generatetodescriptor", "params": ['$1', '$descriptor']}'
	invoke "${JSON_STRING}" 
}
function listunspent {
# curl --user $user:$password --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method":  "listunspent", "params": []}' -H 'content-type: text/plain;' $url | python -mjson.tool
	JSON_STRING='{"jsonrpc": "1.0", "id": "curltest", "method":  "listunspent", "params": []}'
	invoke "${JSON_STRING}" 
}



getdescriptorinfo
createwallet
importdescriptors
# generatetodescriptor 10  
listunspent