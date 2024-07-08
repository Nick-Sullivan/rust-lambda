output "gateway_url" {
  description = "URL for invoking API Gateway."
  value       = aws_api_gateway_stage.gateway.invoke_url
}

output "client_id" {
  description = "Client ID for AWS Cognito."
  value       = aws_cognito_user_pool_client.users.id
}

resource "aws_ssm_parameter" "cognito_pool_id" {
  name  = "${local.prefix_parameter}/Cognito/UserPoolId"
  type  = "String"
  value = aws_cognito_user_pool.users.id
}

resource "aws_ssm_parameter" "cognito_client_id" {
  name  = "${local.prefix_parameter}/Cognito/ClientId"
  type  = "String"
  value = aws_cognito_user_pool_client.users.id
}

resource "aws_ssm_parameter" "automated_tester_password" {
  name  = "${local.prefix_parameter}/AutomatedTester/Password"
  type  = "SecureString"
  value = random_password.automated_tester_password.result
}

resource "aws_ssm_parameter" "automated_tester_username" {
  name  = "${local.prefix_parameter}/AutomatedTester/Username"
  type  = "String"
  value = local.automated_tester_username
}
