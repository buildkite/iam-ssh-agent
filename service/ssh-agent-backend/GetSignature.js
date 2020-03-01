const AWS = require('aws-sdk');

import { fetchKeyParametersListForCaller, fetchPublicKeyForParameter } from './lib.js';

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
        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let { KeyBlob, Data, Flags } = JSON.parse(event.body);

        // Find the parameter that stores the private/public key pair for blob
        // searching the list of keys the caller has access to.
        let keyList = await fetchKeyParametersListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyList.join(',')}`);

        for (const keyParameter of keyList) {
            let publicKey = await fetchPublicKeyForParameter(keyParameter);
            if (publicKey != KeyBlob) {
                continue;
            }

            let privateKey = fetchPrivateKeyForParameter(keyParameter);

            return {
                'statusCode': 200,
                'body': JSON.stringify({
                    signature: "",
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
