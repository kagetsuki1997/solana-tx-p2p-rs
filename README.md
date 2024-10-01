# Project Template

## Project

### Project Structure

- [`dev-support`](.) contains development utilities
  - [`dev-support/bin`](bin) contains tools which will be used through development process
  - [`dev-support/ci-bin`](ci-bin) contains scripts used by CI
  - [`dev-support/containers`](containers) contains the container related definitions
  - [`dev-support/flake-modules`](flake-modules) contains Nix flake modules (ex. development environment)

### Git submodule

```
git submodule update --recursive --remote
```

### Run

run `zta-iam` in local environment

```
ZTA_IAM_APPLICATION_PRIVATE_KEY="-----BEGIN EC PRIVATE KEY-----
MHcCAQEEILmmqBu6cIaGqqzUiwXK+ffJBo+A7+RJKkSwsKIz5WGEoAoGCCqGSM49
AwEHoUQDQgAEcRheC6EErVmslgu/URthkOdZ/5FZvCp32AcC9U/UMv4tEN6yAkrD
ziOOPtX9iFgzvedfNnZC2iegYh8as9/UDw==
-----END EC PRIVATE KEY-----" \
ZTA_IAM_AUTH_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEsz0Ivl3PFbTbfXLM8zyuKL7CEkAWdzrLqlxtAHgLrVk2D4xHAnBB4g5f3rfsvhpWpvEEUWo3oyg/Ik/iOuwBsg==-----END PUBLIC KEY-----" \
ZTA_IAM_CARD_IDP_AUTH_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEcRheC6EErVmslgu/URthkOdZ/5FZ
vCp32AcC9U/UMv4tEN6yAkrDziOOPtX9iFgzvedfNnZC2iegYh8as9/UDw==
-----END PUBLIC KEY-----" cargo run --bin zta-iam -- \
    run \
    --pg-host localhost \
    --pg-port 5432 \
    --pg-user admin \
    --pg-password admin \
    --pg-database my_postgres \
    --keycloak-endpoint http://localhost:8180 \
    --keycloak-realm myrealm \
    --keycloak-client-id zta_iam_service_account \
    --keycloak-client-secret f03V3J4N6x1wBZn4OB9uytzXGWh9qLoK \
    --ip2location-bin-file-path iam/server/IP2LOCATION-LITE-DB5.BIN \
    --card-idp-name card-idp \
    --card-idp-endpoint http://192.168.1.116:9998 \
    --card-idp-client-id zta_iam \
    --card-idp-client-secret 5b3GOY1IrtiAQE0ONcHoZObx3KZTn1xQ \
    --api-public-domain http://localhost:8007 \
    --client-public-key-map "key1=-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEcRheC6EErVmslgu/URthkOdZ/5FZ
vCp32AcC9U/UMv4tEN6yAkrDziOOPtX9iFgzvedfNnZC2iegYh8as9/UDw==
-----END PUBLIC KEY-----,key2=-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAmas2bbG3yIfJwW7TuEdM
TBMjR+uHBIWC6mmxRTzTtpzCavGLp+tkXvkQszgjunNavXPbnlf7HsoroUoekJ7R
k1KV96O8B/IPkWakelwYT937MoTVFlGR2kr4t906nLajN9DNdgiKECHp9UQ2LTHG
4QxrNpAyf3X+d9DkVeka8t+OFa1mA5Fv6sOFJnHteM8iesuMAztG/G2GHlzX0NrQ
N+29rtjUUskhLPpZ5jO2RhJoiYnr7UTycq4bi1QaLqdjzI5c5ORV5oWTRX8/OLU6
6tOCcvjBiCpeutGnYjzT0gQcT2pssigtdzUP5cWPK5D8t7LK0UzQ6899sIwEaiJU
BwIDAQAB
-----END PUBLIC KEY-----"
```

## Contributing

See [CONTRIBUTING.md](dev-support/CONTRIBUTING.md).

## License

See [LICENSE](LICENSE).
