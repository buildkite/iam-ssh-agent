const AWS = require('aws-sdk');

const lib = require('./lib.js');
const ssh2 = require('./node_modules/ssh2-streams/index.js');
const crypto = require('crypto');

async function fetchPrivateKeyForParameter(keyParameter) {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    let response = await ssm.getParameter({
        Name: keyParameter,
        WithDecryption: true,
    }).promise();

    return response.Parameter.Value;
}

exports.handler = async (event, context) => {
    try {
        console.log(`fn=handler event=${JSON.stringify(event)}`);

        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let { pubkey, data, flags } = JSON.parse(event.body);

        let decodedPubkey = Buffer.from(pubkey, 'base64');
        let decodedData = Buffer.from(data, 'base64');

        // Find the parameter that stores the private/public key pair for blob
        // searching the list of keys the caller has access to.
        let keyParameterList = await lib.fetchKeyParameterListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyParameterList.join(',')}`);

        let parameters = await lib.fetchPublicKeyParameters(keyParameterList);

        for (const parameter of parameters) {
            let parsedKey = ssh2.utils.parseKey(parameter.Value);

            // pubkey is base64(pubkey bits)
            // decoded_pubkey is binary key bits
            //
            // key is a string rep of the public key with comment etc
            // parsed_key is an OpenSSH key from ssh2-streams
            if (!decodedPubkey.equals(parsedKey.getPublicSSH())) {
                console.log(`fn=handler caller=${caller} key=${parameter.Name} at=skip`);
                continue;
            }
            console.log(`fn=handler caller=${caller} key=${parameter.Name} at=match`);

            // Depending on the parameter contents ssh2.utils.parseKey might
            // return a single key or a list of keys. We only support one.
            let privateKey = [].concat(ssh2.utils.parseKey(await fetchPrivateKeyForParameter(parameter.Name)))[0];

            var signatureBlob;
            if (privateKey.type == "ssh-rsa") {
                if (flags == 2) {
                    // SSH_AGENT_RSA_SHA2_256
                    signatureBlob = [Buffer.from('rsa-sha2-256'), crypto.sign('sha256', decodedData, privateKey.getPrivatePEM())];
                } else if (flags == 4) {
                    // SSH_AGENT_RSA_SHA2_512
                    signatureBlob = [Buffer.from('rsa-sha2-512'), crypto.sign('sha512', decodedData, privateKey.getPrivatePEM())];
                } else {
                    // SSH_AGENT_RSA_SHA1
                    signatureBlob = [Buffer.from('ssh-rsa'), crypto.sign('sha1', decodedData, privateKey.getPrivatePEM())];
                }
            } else {
                signatureBlob = [Buffer.from(privateKey.type), privateKey.sign(decodedData)];
            }

            let typeLength = Buffer.alloc(4);
            typeLength.writeUInt32BE(signatureBlob[0].length);

            let sigLength = Buffer.alloc(4);
            sigLength.writeUInt32BE(signatureBlob[1].length);

            let encodedSignature = Buffer.concat([typeLength, signatureBlob[0], sigLength, signatureBlob[1]]).toString('base64');

            console.log(`fn=handler caller=${caller} key=${parameter.Name} signature=${encodedSignature}`);

            return {
                'statusCode': 200,
                'body': JSON.stringify({
                    signature: encodedSignature,
                }),
            }
        }
        
        return {
            'statusCode': 404,
            'body': JSON.stringify({
                message: "key blob not found in list of keys caller has access to",
            })
        }
    } catch (err) {
        console.log(err);
        return err;
    }
};
