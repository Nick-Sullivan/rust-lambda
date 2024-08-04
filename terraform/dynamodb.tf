
resource "aws_dynamodb_table" "game" {
  name         = "${local.prefix}Game"
  hash_key     = "id"
  billing_mode = "PAY_PER_REQUEST"
  # stream_enabled   = true
  # stream_view_type = "NEW_AND_OLD_IMAGES"
  attribute {
    name = "id"
    type = "S"
  }
}

resource "aws_dynamodb_table" "websocket_connection" {
  name         = "${local.prefix}Websocket"
  hash_key     = "connection_id"
  billing_mode = "PAY_PER_REQUEST"
  attribute {
    name = "connection_id"
    type = "S"
  }
}

resource "aws_dynamodb_table" "database" {
  name         = local.prefix
  hash_key     = "name"
  billing_mode = "PAY_PER_REQUEST"
  attribute {
    name = "name"
    type = "S"
  }
}
