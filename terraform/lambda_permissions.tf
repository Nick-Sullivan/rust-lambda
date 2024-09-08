
data "aws_iam_policy_document" "lambda_assume_role" {
  statement {
    actions = ["sts:AssumeRole"]
    effect  = "Allow"
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "access_dynamodb" {
  statement {
    actions = [
      "dynamodb:ConditionCheckItem",
      "dynamodb:DeleteItem",
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:Query",
      "dynamodb:Scan",
      "dynamodb:UpdateItem",
    ]
    effect = "Allow"
    resources = [
      aws_dynamodb_table.database.arn,
      aws_dynamodb_table.game.arn,
      aws_dynamodb_table.websocket_connection.arn,
    ]
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

data "aws_iam_policy_document" "put_event" {
  statement {
    actions = [
      "events:PutEvents",
    ]
    effect = "Allow"
    resources = [
      "arn:aws:events:${local.region}:${local.aws_account_id}:event-bus/default",
    ]
  }
}

data "aws_iam_policy_document" "read_sqs" {
  statement {
    actions = [
      "sqs:ReceiveMessage",
      "sqs:DeleteMessage",
      "sqs:GetQueueAttributes",
    ]
    effect = "Allow"
    resources = [
      aws_sqs_queue.websocket_disconnected_queue.arn,
    ]
  }
}
