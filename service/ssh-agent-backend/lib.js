const AWS = require('aws-sdk');

exports.fetchKeyParametersListForCaller = async function (caller) => {
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
