FROM ubuntu:18.04

ADD ./target/release/mlc /bin/mlc
RUN PATH=$PATH:/bin/mlc
