# The static musl binary is built in CI (see .github/workflows/ci.yml) and copied into the image.
# Alpine is about 10x smaller than ubuntu. Its base image already contains the root certificates
# for https links, git is only needed for the --gitignore and --gituntracked options.
FROM alpine:3
RUN apk add --no-cache git
COPY --chmod=755 target/release/mlc /usr/local/bin/mlc
