const AWS = require('aws-sdk');

import { fetchKeyParametersListForCaller, fetchPublicKeyForParameter } from './lib.js';

exports.handler = async (event, context) => {
    try {
        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let keyParameterList = await fetchKeyParameterListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyParameterList.join(',')}`);

        // TODO do a bulk ssm:GetParameters with batches of 10 keys
        let keys = await Promise.all(keyParameterList.map(keyParameter => {
            return fetchPublicKeyForParameter(keyParameter);
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
