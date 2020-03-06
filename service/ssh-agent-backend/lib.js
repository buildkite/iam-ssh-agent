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

function replaceKeyCommentWithParameterArn(key, arn) {
    let components = key.split(' ');
    if (components.length == 3) {
        components.pop();
    }
    components.push(arn);

    return components.join(' ');
}

function chunkArrayInGroups(arr, chunkSize) {
    var groups = [], i;
    for (i = 0; i < arr.length; i += chunkSize) {
        groups.push(arr.slice(i, i + chunkSize));
    }
    return groups;
}

exports.fetchPublicKeyParameters = async (keyParameters) => {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    // Fetch up to 10 parameters at a time
    var groups = chunkArrayInGroups(keyParameters.map(keyParameter => `${keyParameter}.pub`), 10);

    let responses = await Promise.all(groups.map(async (keyParameters) => {
        let response = await ssm.getParameters({
            Names: keyParameters,
        }).promise();
        
        return response.Parameters;
    }));

    return responses
        .flat()
        .map((parameter) => {
            return {
                // Transform the public key paramter name back to the private key parameter name
                Name: parameter.Name.slice(0, -4),
                Value: replaceKeyCommentWithParameterArn(parameter.Value, parameter.ARN),
            }
        });
};
