FROM ubuntu:24.04

RUN apt-get update; apt-get install -y ca-certificates; update-ca-certificates
RUN apt-get install git -y
ADD ./target/release/mlc /bin/mlc
RUN chmod +x /bin/mlc
RUN PATH=$PATH:/bin/mlc
