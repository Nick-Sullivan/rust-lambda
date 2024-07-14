
resource "aws_cloudwatch_log_group" "monolith" {
  name              = "/aws/lambda/${local.prefix}"
  retention_in_days = 90
}

resource "aws_lambda_function" "monolith" {
  package_type  = "Image"
  image_uri     = "${aws_ecr_repository.lambda.repository_url}@${data.aws_ecr_image.lambda.id}"
  function_name = local.prefix
  role          = aws_iam_role.monolith.arn
  timeout       = 5
  depends_on = [
    aws_cloudwatch_log_group.monolith,
    terraform_data.lambda_push,
  ]
  environment {
    variables = {
      HANDLER_TYPE    = "hello",
      TABLE_NAME      = aws_dynamodb_table.storage.name,
      REGION_NAME     = local.region,
      API_GATEWAY_URL = aws_apigatewayv2_stage.websocket.invoke_url,
    }
  }
}

resource "aws_iam_role" "monolith" {
  name                = local.prefix
  description         = "Allows Lambda run"
  assume_role_policy  = data.aws_iam_policy_document.lambda_assume_role.json
  managed_policy_arns = ["arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"]
  inline_policy {
    name   = "DynamoWriteAccess"
    policy = data.aws_iam_policy_document.access_dynamodb.json
  }
  inline_policy {
    name   = "ApiGatewayConnections"
    policy = data.aws_iam_policy_document.api_connections.json
  }
}

data "aws_iam_policy_document" "api_connections" {
  # Allow Lambda to send messages to API gateway connections
  statement {
    actions = [
      "execute-api:ManageConnections",
    ]
    effect    = "Allow"
    resources = ["arn:aws:execute-api:${local.region}:${local.aws_account_id}:${aws_apigatewayv2_api.websocket.id}/*"]
  }
}
