FROM ubuntu:18.04
MAINTAINER Yu Ding

ENV DEBIAN_FRONTEND=noninteractive
ENV rust_toolchain  nightly-2020-10-25
ENV sdk_bin         https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.9.101.2.bin

RUN apt-get update && \
    apt-get install -y gnupg2 apt-transport-https ca-certificates curl software-properties-common build-essential automake autoconf libtool protobuf-compiler libprotobuf-dev git-core libprotobuf-c0-dev cmake pkg-config expect gdb

RUN curl -fsSL https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add - && \
    add-apt-repository "deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main" && \
    apt-get update  && \
    apt-get install -y  libsgx-urts libsgx-dcap-ql \
        libsgx-enclave-common-dbgsym libsgx-dcap-ql-dbgsym && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /var/cache/apt/archives/* && \
    mkdir /var/run/aesmd && \
    mkdir /etc/init

RUN curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
    chmod +x /root/rustup-init && \
    echo '1' | /root/rustup-init --default-toolchain ${rust_toolchain} && \
    echo 'source /root/.cargo/env' >> /root/.bashrc && \
    /root/.cargo/bin/rustup component add rust-src rls rust-analysis clippy rustfmt && \
    /root/.cargo/bin/cargo install xargo && \
    rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

RUN mkdir /root/sgx && \
    curl --output /root/sgx/sdk.bin ${sdk_bin} && \
    cd /root/sgx && \
    chmod +x /root/sgx/sdk.bin && \
    echo -e 'no\n/opt' | /root/sgx/sdk.bin && \
    echo 'source /opt/sgxsdk/environment' >> /root/.bashrc && \
    echo 'alias start-aesm="LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm /opt/intel/sgx-aesm-service/aesm/aesm_service"' >> /root/.bashrc && \
    rm -rf /root/sgx*

RUN cd /usr/lib/x86_64-linux-gnu && \
    ln -s libsgx_dcap_ql.so.1 libsgx_dcap_ql.so

WORKDIR /root
# FROM ubuntu:18.04

# ENV DEBIAN_FRONTEND=noninteractive

# RUN apt update && apt install -y autoconf automake bison build-essential cmake curl dpkg-dev expect flex gcc-8 gdb git git-core gnupg kmod libboost-system-dev libboost-thread-dev libcurl4-openssl-dev libiptcdata0-dev libjsoncpp-dev liblog4cpp5-dev libprotobuf-c0-dev libprotobuf-dev libssl-dev libtool libxml2-dev ocaml ocamlbuild pkg-config protobuf-compiler python texinfo uuid-dev vim wget dkms gnupg2 apt-transport-https software-properties-common && \
#     rm -rf /var/lib/apt/lists/* && \
#     rm -rf /var/cache/apt/archives/*

# ENV BINUTILS_DIST="ubuntu18.04"
# #ENV BINUTILS_DIST="SELF_BUILT"
# ENV LD_LIBRARY_PATH=/usr/lib:/usr/local/lib
# ENV LD_RUN_PATH=/usr/lib:/usr/local/lib

# ADD 02_binutils.sh /root
# RUN bash /root/02_binutils.sh

# ENV SDK_DIST="INTEL_BUILT"
# ENV SDK_URL="https://download.01.org/intel-sgx/sgx-linux/2.12/distro/ubuntu18.04-server/sgx_linux_x64_sdk_2.12.100.3.bin"
# #ENV SDK_DIST="SELF_BUILT"
# ADD 03_sdk.sh /root
# RUN bash /root/03_sdk.sh
    
# # Sixth, PSW

# ENV CODENAME        bionic
# ENV VERSION         2.12.100.3-bionic1
# ENV DCAP_VERSION    1.9.100.3-bionic1

# ADD 04_psw.sh /root
# RUN bash /root/04_psw.sh

# # Seventh, Rust

# ENV rust_toolchain  nightly-2020-10-25
# ADD 05_rust.sh /root
# RUN bash /root/05_rust.sh

# ENV DEBIAN_FRONTEND=
# ENV CODENAME=
# ENV VERSION=

# WORKDIR /root
