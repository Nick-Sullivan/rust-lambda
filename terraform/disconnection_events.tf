
# Disconnection events trigger a rule

resource "aws_cloudwatch_event_rule" "websocket_disconnected" {
  name          = "${local.prefix}-WebsocketDisconnected"
  description   = "A player has disconnected from the websocket"
  event_pattern = <<-EOF
    {
      "source": ["${local.prefix}.Websocket"],
      "detail-type": ["Disconnected"]
    }
  EOF
}

# Log the event

# resource "aws_cloudwatch_log_group" "websocket_disconnected" {
#   name              = "/aws/events/${aws_cloudwatch_event_rule.websocket_disconnected.name}"
#   retention_in_days = 90
# }

# resource "aws_cloudwatch_event_target" "websocket_disconnected_logs" {
#   rule      = aws_cloudwatch_event_rule.websocket_disconnected.name
#   target_id = "SendToCloudWatch"
#   arn       = aws_cloudwatch_log_group.websocket_disconnected.arn
#   retry_policy {
#     maximum_retry_attempts       = 0
#     maximum_event_age_in_seconds = 24 * 60 * 60
#   }
# }

# Send the event to an SQS queue to act as a timer

resource "aws_cloudwatch_event_target" "websocket_disconnected_queue" {
  rule      = aws_cloudwatch_event_rule.websocket_disconnected.name
  target_id = "AddToSqs"
  arn       = aws_sqs_queue.websocket_disconnected_queue.arn
  retry_policy {
    maximum_retry_attempts       = 0
    maximum_event_age_in_seconds = 60
  }
}

resource "aws_sqs_queue" "websocket_disconnected_queue" {
  name                      = "${local.prefix}-WebsocketDisconnection"
  delay_seconds             = 60
  message_retention_seconds = 6 * 60 * 60
}

resource "aws_sqs_queue_policy" "websocket_disconnected_queue" {
  queue_url = aws_sqs_queue.websocket_disconnected_queue.id
  policy    = data.aws_iam_policy_document.websocket_disconnected_queue.json
}

data "aws_iam_policy_document" "websocket_disconnected_queue" {
  statement {
    actions = [
      "sqs:SendMessage",
    ]
    effect = "Allow"
    resources = [
      aws_sqs_queue.websocket_disconnected_queue.arn,
    ]
    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }
    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [aws_cloudwatch_event_rule.websocket_disconnected.arn]
    }
  }
}

resource "aws_lambda_event_source_mapping" "check_session_timeout" {
  event_source_arn = aws_sqs_queue.websocket_disconnected_queue.arn
  function_name    = aws_lambda_function.sqs.function_name
  batch_size       = 1
}
