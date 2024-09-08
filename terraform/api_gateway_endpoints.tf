# Order of creation is important
# method -> integration -> method response -> integration response

locals {
  # this determines when to redeploy API gateway
  all_integrations = [
    aws_api_gateway_method.hello,
    aws_api_gateway_method.goodbye,

    aws_api_gateway_integration.hello,
    aws_api_gateway_integration.goodbye,

    aws_api_gateway_method_response.hello_200,
    aws_api_gateway_method_response.goodbye_200,
  ]
}


# hello

resource "aws_api_gateway_resource" "hello" {
  path_part   = "hello"
  parent_id   = aws_api_gateway_rest_api.gateway.root_resource_id
  rest_api_id = aws_api_gateway_rest_api.gateway.id
}

resource "aws_api_gateway_method" "hello" {
  rest_api_id   = aws_api_gateway_rest_api.gateway.id
  resource_id   = aws_api_gateway_resource.hello.id
  http_method   = "POST"
  authorization = "COGNITO_USER_POOLS"
  authorizer_id = aws_api_gateway_authorizer.gateway.id
}

resource "aws_api_gateway_integration" "hello" {
  rest_api_id             = aws_api_gateway_rest_api.gateway.id
  resource_id             = aws_api_gateway_resource.hello.id
  http_method             = aws_api_gateway_method.hello.http_method
  uri                     = aws_lambda_function.http.invoke_arn
  content_handling        = "CONVERT_TO_TEXT"
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
}

resource "aws_api_gateway_method_response" "hello_200" {
  rest_api_id = aws_api_gateway_rest_api.gateway.id
  resource_id = aws_api_gateway_resource.hello.id
  http_method = aws_api_gateway_integration.hello.http_method
  status_code = "200"
  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}

# goodbye


resource "aws_api_gateway_resource" "goodbye" {
  path_part   = "goodbye"
  parent_id   = aws_api_gateway_rest_api.gateway.root_resource_id
  rest_api_id = aws_api_gateway_rest_api.gateway.id
}

resource "aws_api_gateway_method" "goodbye" {
  rest_api_id   = aws_api_gateway_rest_api.gateway.id
  resource_id   = aws_api_gateway_resource.goodbye.id
  http_method   = "POST"
  authorization = "COGNITO_USER_POOLS"
  authorizer_id = aws_api_gateway_authorizer.gateway.id
}

resource "aws_api_gateway_integration" "goodbye" {
  rest_api_id             = aws_api_gateway_rest_api.gateway.id
  resource_id             = aws_api_gateway_resource.goodbye.id
  http_method             = aws_api_gateway_method.goodbye.http_method
  uri                     = aws_lambda_function.http.invoke_arn
  content_handling        = "CONVERT_TO_TEXT"
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
}

resource "aws_api_gateway_method_response" "goodbye_200" {
  rest_api_id = aws_api_gateway_rest_api.gateway.id
  resource_id = aws_api_gateway_resource.goodbye.id
  http_method = aws_api_gateway_integration.goodbye.http_method
  status_code = "200"
  response_parameters = {
    "method.response.header.Access-Control-Allow-Headers" = true
    "method.response.header.Access-Control-Allow-Methods" = true
    "method.response.header.Access-Control-Allow-Origin"  = true
  }
}
