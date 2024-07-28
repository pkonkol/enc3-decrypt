FROM ubuntu:20.04 AS build

ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/London

RUN apt-get update && apt-get -y install tzdata
RUN apt-get -y install zlibc zlib1g-dev zlib1g cmake gcc g++ make

COPY cmake /usr/src/otcv8decrypt/cmake/
COPY src /usr/src/otcv8decrypt/src/
COPY CMakeLists.txt /usr/src/otcv8decrypt/
WORKDIR /usr/src/otcv8decrypt/build
RUN cmake .. && make

FROM ubuntu:20.04

# RUN apt-get update && apt-get -y install tzdata
# RUN apt-get -y install zlibc zlib1g-dev zlib1g cmake gcc g++ make

COPY --from=build /usr/src/otcv8decrypt/build/otcv8decrypt /bin/otcv8decrypt
# COPY init.lua /bin/init.lua

# ENTRYPOINT ["/bin/otcv8decrypt"]
WORKDIR /workdir/test
ENTRYPOINT ["/bin/bash"]
