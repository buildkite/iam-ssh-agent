const AWS = require('aws-sdk');

import fetchKeyParametersListForCaller from './lib.js';

// TODO do a bulk GetParameters with batches of 10 keys
async function fetchPublicKey(key) {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    let response = await ssm.getParameter({
        Name: key
    }).promise();

    return response.Parameter.Value;
}

exports.handler = async (event, context) => {
    try {
        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let keyList = await fetchKeyParametersListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyList.join(',')}`);

        let keys = await Promise.all(keyList.map(key => {
            let publicKey = `${key}.pub`;
            return fetchPublicKey(publicKey);
        }));
        
        return {
            'statusCode': 200,
            'body': JSON.stringify({
                identities: keys,
            })
        }
    } catch (err) {
        console.log(err);
        return err;
    }
};
