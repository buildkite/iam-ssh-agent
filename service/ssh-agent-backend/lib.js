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

async function fetchPublicKeyForParameter(keyParameter) {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    let response = await ssm.getParameter({
        Name: `${keyParameter}.pub`,
    }).promise();

    return replaceKeyCommentWithParameterArn(response.Parameter.Value, response.Parameter.ARN);
}

exports.fetchPublicKeyForParameter = fetchPublicKeyForParameter;

function* chunkArrayInGroups(arr, size) {
  for (var i=0; i<arr.length; i+=size)
    yield arr.slice(i, i+size);
}

exports.fetchPublicKeysForParameters = async (keyParameters) => {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    // Fetch up to 10 parameters at a time
    let groups = chunkArrayInGroups(keyParameters.map((keyParameter) => {
        `${keyParameter}.pub`
    }), 10);

    let responses = await Promise.all(groups.map((keyParameters) => {
        return ssm.getParameters({
            Names: keyParameters,
        })
    }));

    let parameters = responses
        .map((response) => {
            response.Parameters
        })
        .flat();

    return parameters.map((parameter) => {
        return replaceKeyCommentWithParameterArn(parameter.Value, parameter.ARN);
    });
};
