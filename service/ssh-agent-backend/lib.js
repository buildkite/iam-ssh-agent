const AWS = require('aws-sdk');

exports.fetchKeyParameterListForCaller = async (caller) => {
    let dynamodb = new AWS.DynamoDB({apiVersion: '2012-08-10'});
    
    let response = await dynamodb.getItem({
        TableName: process.env.KEY_PERMISSIONS_TABLE_NAME,
        Key: {
            "IamEntityUniqueId": {
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
};

async function fetchPublicKeyForParameter(keyParameter) {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    let response = await ssm.getParameter({
        Name: `${keyParameter}.pub`,
    }).promise();

    let value = response.Parameter.Value;

    let components = value.split(' ');
    if (components.length == 3) {
        components.pop();
    }
    components.push(response.Parameter.ARN);

    return components.join(' ');
}

exports.fetchPublicKeyForParameter = fetchPublicKeyForParameter;

exports.fetchPublicKeysForParameters = async (keyParameters) => {
    Promise.all(keyParameters.map((keyParameter) => {
        fetchPublicKeyForParameter(keyParameter)
    }));
};
