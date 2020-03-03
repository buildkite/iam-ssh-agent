const AWS = require('aws-sdk');

const lib = require('./lib.js');
const ssh2 = require('./node_modules/ssh2-streams/index.js');

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

        let decoded_pubkey = Buffer.from(pubkey, 'base64');
        let decoded_data = Buffer.from(data, 'base64');

        // Find the parameter that stores the private/public key pair for blob
        // searching the list of keys the caller has access to.
        let keyList = await lib.fetchKeyParameterListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyList.join(',')}`);

        for (const keyParameter of keyList) {
            let key = await lib.fetchPublicKeyForParameter(keyParameter);
            let parsed_key = ssh2.utils.parseKey(key);

            // pubkey is base64(pubkey bits)
            // decoded_pubkey is binary key bits
            //
            // key is a string rep of the public key with comment etc
            // parsed_key is an OpenSSH key from ssh2-streams
            if (!decoded_pubkey.equals(parsed_key.getPublicSSH())) {
                console.log(`fn=handler caller=${caller} key=${keyParameter} at=skip`);
                continue;
            }
            console.log(`fn=handler caller=${caller} key=${keyParameter} at=match`);

            // Depending on the parameter contents ssh2.utils.parseKey might
            // return a single key or a list of keys. We only support one.
            let privateKey = [].concat(ssh2.utils.parseKey(await fetchPrivateKeyForParameter(keyParameter)))[0];
            let signature = privateKey.sign(decoded_data);

            console.log(`fn=handler caller=${caller} key=${keyParameter} signature=${signature}`);

            return {
                'statusCode': 200,
                'body': JSON.stringify({
                    signature: signature.toString('base64'),
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
