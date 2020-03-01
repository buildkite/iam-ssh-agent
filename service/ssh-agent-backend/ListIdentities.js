const AWS = require('aws-sdk');

async function listKeysForCaller(caller) {
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



    let key = await fetchPublicKey('/github/keithduncan/sara-r4/deploy-key.pub');
}

async function fetchPublicKey(key) {
    let ssm = new AWS.SSM({apiVersion: '2014-11-06'});

    let parameter = await ssm.getParameter({
        Name: key
    }).promise();



    return parameter;
}

/**
 *
 * Event doc: https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
 * @param {Object} event - API Gateway Lambda Proxy Input Format
 *
 * Context doc: https://docs.aws.amazon.com/lambda/latest/dg/nodejs-prog-model-context.html 
 * @param {Object} context
 *
 * Return doc: https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html
 * @returns {Object} object - API Gateway Lambda Proxy Output Format
 * 
 */
exports.handler = async (event, context) => {
    try {
        let identity = event.requestContext.identity;

        let keys = await listKeysForCaller(caller);
        
        return {
            'statusCode': 200,
            'body': JSON.stringify({
                message: key,
                event: event,
                context_identity: context.identity,
                // location: ret.data.trim()
            })
        }
    } catch (err) {
        console.log(err);
        return err;
    }
};
