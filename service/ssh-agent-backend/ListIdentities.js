const AWS = require('aws-sdk');

async function fetchKeyParametersListForCaller(caller) {
    let dynamodb = new AWS.DynamoDB({apiVersion: '2012-08-10'});
    
    let response = await dynamodb.getItem({
        TableName: process.env.KEY_PERMISSIONS_TABLE_NAME,
        Key: {
            "IamEntityArn": {
                S: caller,
            }
        },
        AttributesToGet: [
            "Parameters",
        ],
    }).promise();

    let item = response.Item;
    if (item  == undefined) {
        return [];
    }

    let parameters = item.Parameters;
    if (parameters == undefined) {
        return [];
    }

    return parameters.SS;
}

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

        let keyList = await fetchKeyParametersListForCaller(caller);

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
