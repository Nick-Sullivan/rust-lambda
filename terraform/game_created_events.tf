

# Disconnection events trigger a rule

resource "aws_cloudwatch_event_rule" "game_created" {
  name          = "${local.prefix}-GameCreated"
  description   = "A game has been created"
  event_pattern = <<-EOF
    {
      "source": ["${local.prefix}.GameCreated"],
      "detail-type": ["Game created"]
    }
  EOF
}

# Allow all eventbridge rules to write to cloudwatch logs

resource "aws_cloudwatch_log_resource_policy" "eventbridge_logs" {
  policy_name = "${local.prefix}-EventBridgeLogs"
  policy_document = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Principal = {
          Service = ["events.amazonaws.com", "delivery.logs.amazonaws.com"]
        }
        Resource = "arn:aws:logs:${local.region}:${local.aws_account_id}:log-group:/aws/events/*:*"
      }
    ]
  })
}

# Log the event

# resource "aws_cloudwatch_event_target" "game_created_logs" {
#   rule      = aws_cloudwatch_event_rule.game_created.name
#   target_id = "SendToCloudWatch"
#   arn       = aws_cloudwatch_log_group.game_created.arn
#   retry_policy {
#     maximum_retry_attempts       = 0
#     maximum_event_age_in_seconds = 24 * 60 * 60
#   }
# }

# resource "aws_cloudwatch_log_group" "game_created" {
#   name              = "/aws/events/${aws_cloudwatch_event_rule.game_created.name}"
#   retention_in_days = 90
# }

# Email me about a new game

resource "aws_cloudwatch_event_target" "game_created_email" {
  rule      = aws_cloudwatch_event_rule.game_created.name
  target_id = "SendToAdmin"
  arn       = aws_sns_topic.admin_email.arn
  input_transformer {
    input_paths = {
      game_id = "$.detail.game_id"
    }
    input_template = "\"A new game was created: <game_id>\""
  }
  retry_policy {
    maximum_retry_attempts       = 0
    maximum_event_age_in_seconds = 24 * 60 * 60
  }
}

resource "aws_sns_topic" "admin_email" {
  name = "${local.prefix}-NotifyAdmin"
}

resource "aws_sns_topic_policy" "admin_email" {
  arn    = aws_sns_topic.admin_email.arn
  policy = data.aws_iam_policy_document.admin_email.json
}

data "aws_iam_policy_document" "admin_email" {
  statement {
    effect = "Allow"
    actions = [
      "SNS:Publish",
    ]
    principals {
      type        = "Service"
      identifiers = ["events.amazonaws.com"]
    }
    resources = [
      aws_sns_topic.admin_email.arn,
    ]
  }
}

resource "aws_sns_topic_subscription" "admin_email" {
  topic_arn = aws_sns_topic.admin_email.arn
  protocol  = "email"
  endpoint  = local.admin_email
}
