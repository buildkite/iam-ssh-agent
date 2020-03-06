const AWS = require('aws-sdk');

const lib = require('./lib.js');

exports.handler = async (event, context) => {
    try {
        let identity = event.requestContext.identity;
        let [caller,_] = identity.caller.split(":");
        console.log(`fn=handler caller=${caller}`);

        let keyParameterList = await lib.fetchKeyParameterListForCaller(caller);
        console.log(`fn=handler caller=${caller} keys=${keyParameterList.join(',')}`);

        let keys = await lib.fetchPublicKeyParameters(keyParameterList);
        
        return {
            'statusCode': 200,
            'body': JSON.stringify({
                identities: keys.map(parameter => parameter.Value),
            })
        }
    } catch (err) {
        console.log(err);
        return err;
    }
};
