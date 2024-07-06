

# resource "aws_cloudwatch_log_group" "all" {
#   for_each          = local.lambda_names
#   name              = "/aws/lambda/${each.value}"
#   retention_in_days = 90
# }

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
      HANDLER_TYPE = "hello"
    }
  }
}

resource "aws_iam_role" "monolith" {
  name                = local.prefix
  description         = "Allows Lambda run"
  assume_role_policy  = data.aws_iam_policy_document.lambda_assume_role.json
  managed_policy_arns = ["arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"]
}
