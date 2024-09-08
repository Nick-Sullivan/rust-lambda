
resource "aws_lambda_function" "http" {
  package_type  = "Image"
  image_uri     = "${aws_ecr_repository.lambda.repository_url}@${data.aws_ecr_image.lambda.id}"
  function_name = "${local.prefix}-HTTP"
  role          = aws_iam_role.lambda.arn
  timeout       = 5
  image_config {
    entry_point = ["/api_cloud_entry"]
  }
  depends_on = [
    aws_cloudwatch_log_group.http,
    terraform_data.lambda_push,
  ]
  environment {
    variables = {
      DATABASE             = aws_dynamodb_table.database.name,
      WEBSOCKET_TABLE_NAME = aws_dynamodb_table.websocket_connection.name,
      GAME_TABLE_NAME      = aws_dynamodb_table.game.name,
      REGION_NAME          = local.region,
      API_GATEWAY_URL      = aws_apigatewayv2_stage.websocket.invoke_url,
    }
  }
}

resource "aws_lambda_function" "sqs" {
  package_type  = "Image"
  image_uri     = "${aws_ecr_repository.lambda.repository_url}@${data.aws_ecr_image.lambda.id}"
  function_name = "${local.prefix}-SQS"
  role          = aws_iam_role.lambda.arn
  timeout       = 5
  image_config {
    entry_point = ["/api_sqs_entry"]
  }
  depends_on = [
    aws_cloudwatch_log_group.sqs,
    terraform_data.lambda_push,
  ]
  environment {
    variables = {
      DATABASE             = aws_dynamodb_table.database.name,
      WEBSOCKET_TABLE_NAME = aws_dynamodb_table.websocket_connection.name,
      GAME_TABLE_NAME      = aws_dynamodb_table.game.name,
      REGION_NAME          = local.region,
      API_GATEWAY_URL      = aws_apigatewayv2_stage.websocket.invoke_url,
    }
  }
}

resource "aws_iam_role" "lambda" {
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
  inline_policy {
    name   = "EventPublishing"
    policy = data.aws_iam_policy_document.put_event.json
  }
  inline_policy {
    name   = "ReadSqs"
    policy = data.aws_iam_policy_document.read_sqs.json
  }
}

resource "aws_cloudwatch_log_group" "http" {
  name              = "/aws/lambda/${local.prefix}-HTTP"
  retention_in_days = 90
}

resource "aws_cloudwatch_log_group" "sqs" {
  name              = "/aws/lambda/${local.prefix}-SQS"
  retention_in_days = 90
}
