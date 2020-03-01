const AWS = require('aws-sdk');

import { fetchKeyParametersListForCaller, fetchPublicKey } from './lib.js';

exports.handler = async (event, context) => {
    try {
        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let keyList = await fetchKeyParametersListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyList.join(',')}`);

        // TODO do a bulk GetParameters with batches of 10 keys
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
