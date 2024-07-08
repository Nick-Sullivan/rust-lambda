#
# py -3.11 -m venv .venv
# ./.venv/Scripts/activate
# python -m pip install --upgrade pip
# pip install boto3 pycognito

import boto3
from pycognito import Cognito

ENV = "dev"


def main():
    secrets = load_secrets()
    aws = Cognito(
        secrets["POOL_ID"],
        secrets["CLIENT_ID"],
        username=secrets["USERNAME"],
    )
    aws.authenticate(password=secrets["PASSWORD"])
    id_token = aws.id_token
    print(f"id_token: {id_token}")


def load_secrets():
    secrets = {}
    prefix = f"/RustLambda/{ENV.capitalize()}"
    parameter_names = {
        f"{prefix}/AutomatedTester/Username": "USERNAME",
        f"{prefix}/AutomatedTester/Password": "PASSWORD",
        f"{prefix}/Cognito/UserPoolId": "POOL_ID",
        f"{prefix}/Cognito/ClientId": "CLIENT_ID",
    }
    ssm_client = boto3.client("ssm")
    parameters = ssm_client.get_parameters(
        Names=list(parameter_names), WithDecryption=True
    )
    for parameter in parameters["Parameters"]:
        secrets[parameter_names[parameter["Name"]]] = parameter["Value"]
    return secrets


if __name__ == "__main__":
    main()
