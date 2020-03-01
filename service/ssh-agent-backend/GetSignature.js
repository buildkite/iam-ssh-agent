const AWS = require('aws-sdk');

import fetchKeyParametersListForCaller from './lib.js';

async function fetchPrivateKey(key) {
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
        
        return {
            'statusCode': 200,
            'body': JSON.stringify({
                signature: "",
            })
        }
    } catch (err) {
        console.log(err);
        return err;
    }
};
