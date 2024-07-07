
resource "aws_dynamodb_table" "storage" {
  name         = local.prefix
  hash_key     = "name"
  billing_mode = "PAY_PER_REQUEST"
  attribute {
    name = "name"
    type = "S"
  }
}
