ARG BASE_IMAGE=jessie

# skopeo inspect docker://docker.io/library/debian:jessie
FROM docker.io/library/debian@sha256:32ad5050caffb2c7e969dac873bce2c370015c2256ff984b70c1c08b3a2816a0 AS jessie
# This is the last published snapshot of Jessie.
ARG DEBIAN_SNAPSHOT=20230322T152120Z
ENV HOME=/build \
    SHELL=/bin/bash \
    USER=build \
    LOGNAME=build \
    HOSTNAME=builder \
    DEBIAN_FRONTEND=noninteractive
CMD ["/bin/bash", "--login"]
WORKDIR '/build'

ADD scripts/secure-download.sh /usr/local/bin/
ADD scripts/install-sccache.sh /usr/local/bin/
ADD scripts/sccache-wrapper.sh /usr/local/bin/

RUN <<EOF
#!/bin/bash
set -euo pipefail

groupadd -g 1000 build
useradd -u 1000 -g 1000 -d /build -s /bin/bash -m build
chown -R build:build /build /etc/ld.so.conf.d /usr/local

for s in debian_jessie debian_jessie-updates debian-security_jessie/updates; do
  echo "deb http://snapshot.debian.org/archive/${s%_*}/${DEBIAN_SNAPSHOT}/ ${s#*_} main";
done > /etc/apt/sources.list

( echo 'quiet "true";';
  echo 'APT::Get::Assume-Yes "true";';
  echo 'APT::Install-Recommends "false";';
  echo 'Acquire::Check-Valid-Until "false";';
  echo 'Acquire::Retries "5";';
) > /etc/apt/apt.conf.d/99portable-clang

dpkg --add-architecture i386
apt-get update
apt-get install --force-yes ca-certificates curl
install-sccache.sh
EOF

# skopeo inspect docker://docker.io/library/debian:stretch
FROM docker.io/library/debian@sha256:c5c5200ff1e9c73ffbf188b4a67eb1c91531b644856b4aefe86a58d2f0cb05be AS stretch
# This is the last snapshot containing stretch.
ARG DEBIAN_SNAPSHOT=20230423T032736Z

ENV HOME=/build \
    SHELL=/bin/bash \
    USER=build \
    LOGNAME=build \
    HOSTNAME=builder \
    DEBIAN_FRONTEND=noninteractive
CMD ["/bin/bash", "--login"]
WORKDIR '/build'

ADD scripts/secure-download.sh /usr/local/bin/
ADD scripts/install-sccache.sh /usr/local/bin/
ADD scripts/sccache-wrapper.sh /usr/local/bin/

RUN <<EOF
#!/bin/bash
set -euo pipefail

groupadd -g 1000 build
useradd -u 1000 -g 1000 -d /build -s /bin/bash -m build
chown -R build:build /build /etc/ld.so.conf.d /usr/local

for s in debian_stretch debian_stretch-updates debian-security_stretch/updates; do
  echo "deb http://snapshot.debian.org/archive/${s%_*}/${DEBIAN_SNAPSHOT}/ ${s#*_} main";
done > /etc/apt/sources.list

( echo 'quiet "true";';
  echo 'APT::Get::Assume-Yes "true";';
  echo 'APT::Install-Recommends "false";';
  echo 'Acquire::Check-Valid-Until "false";';
  echo 'Acquire::Retries "5";';
) > /etc/apt/apt.conf.d/99portable-clang

apt-get update
apt-get install --force-yes ca-certificates curl
install-sccache.sh
EOF

# skopeo inspect docker://docker.io/library/debian:bullseye
FROM docker.io/library/debian@sha256:f58b816c2acc96e3e1dfe6b6c166c3d52b5541571ac4fa72a64a0bb5beaf25a3 AS bullseye
ARG DEBIAN_SNAPSHOT=20250510T145612Z

ENV HOME=/build \
    SHELL=/bin/bash \
    USER=build \
    LOGNAME=build \
    HOSTNAME=builder \
    DEBIAN_FRONTEND=noninteractive
CMD ["/bin/bash", "--login"]
WORKDIR '/build'

ADD scripts/secure-download.sh /usr/local/bin/
ADD scripts/install-sccache.sh /usr/local/bin/
ADD scripts/sccache-wrapper.sh /usr/local/bin/

RUN <<EOF
#!/bin/bash
set -euo pipefail

groupadd -g 1000 build
useradd -u 1000 -g 1000 -d /build -s /bin/bash -m build
chown -R build:build /build /etc/ld.so.conf.d /usr/local

for s in debian_bullseye debian-security_bullseye-security debian_bullseye-updates; do
  echo "deb http://snapshot.debian.org/archive/${s%_*}/${DEBIAN_SNAPSHOT}/ ${s#*_} main";
done > /etc/apt/sources.list

( echo 'quiet "true";';
  echo 'APT::Get::Assume-Yes "true";';
  echo 'APT::Install-Recommends "false";';
  echo 'Acquire::Check-Valid-Until "false";';
  echo 'Acquire::Retries "5";';
) > /etc/apt/apt.conf.d/99portable-clang

apt-get update
apt-get install --force-yes ca-certificates curl
install-sccache.sh
EOF

FROM ${BASE_IMAGE} AS gnu-build
RUN <<EOF
apt-get install --force-yes autoconf automake bison build-essential gawk libtool gcc make texinfo xz-utils unzip
if [ $(uname -m) = "x86_64" ]; then
  apt-get install --force-yes gcc-multilib
fi
EOF
USER build:build

# Binutils 2.42 has issues compiling on legacy GCC. So we need to bootstrap with 2.41.
FROM gnu-build AS binutils-initial
ARG BINUTILS_INITIAL_VERSION=2.41
ARG BINUTILS_INITIAL_SHA256=ae9a5789e23459e59606e6714723f2d3ffc31c03174191ef0d015bdf06007450

ADD --link --checksum=sha256:${BINUTILS_INITIAL_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/binutils/binutils-${BINUTILS_INITIAL_VERSION}.tar.xz binutils.tar.xz
COPY --link scripts/binutils-build.sh /build/
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    BUILD_CC=/usr/bin/gcc BUILD_CXX=/usr/bin/g++ /build/binutils-build.sh

FROM gnu-build AS gcc-10
ARG PARALLEL=16
ARG GCC_VERSION=10.3.0
ARG GCC_SHA256=64f404c1a650f27fc33da242e1f2df54952e3963a49e06e73f6940f3223ac344
ARG GMP_VERSION=6.1.2
ARG GMP_SHA256=87b565e89a9a684fe4ebeeddb8399dce2599f9c9049854ca8c0dfbdea0e21912
ARG ISL_VERSION=0.18
ARG ISL_SHA256=6b8b0fd7f81d0a957beb3679c81bbb34ccc7568d5682844d8924424a0dadcb1b
ARG MPC_VERSION=1.0.3
ARG MPC_SHA256=617decc6ea09889fb08ede330917a00b16809b8db88c29c31bfbb49cbf88ecc3
ARG MPFR_VERSION=3.1.6
ARG MPFR_SHA256=cf4f4b2d80abb79e820e78c8077b6725bbbb4e8f41896783c899087be0e94068

ADD --link --checksum=sha256:${GCC_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.xz gcc-10.tar.xz
ADD --link --checksum=sha256:${GMP_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/gmp/gmp-${GMP_VERSION}.tar.xz gmp-6.1.tar.xz
ADD --link --checksum=sha256:${ISL_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/isl-${ISL_VERSION}.tar.bz2 isl-0.18.tar.bz2
ADD --link --checksum=sha256:${MPC_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/mpc-${MPC_VERSION}.tar.gz mpc-1.0.tar.gz
ADD --link --checksum=sha256:${MPFR_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/mpfr-${MPFR_VERSION}.tar.bz2 mpfr-3.1.tar.bz2
COPY --from=binutils-initial --link /build/out/toolchain /toolchain/
COPY --link scripts/gcc-build-linux.sh /build/gcc-build.sh
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    GCC_VERSION=10 GMP_VERSION=6.1 ISL_VERSION=0.18 MPC_VERSION=1.0 MPFR_VERSION=3.1 \
    BUILD_CC=/usr/bin/gcc BUILD_CXX=/usr/bin/g++ \
    EXTRA_CONFIGURE_ARGS=--disable-bootstrap \
    PARALLEL=${PARALLEL} \
    /build/gcc-build.sh

# We need to build binutils with a modern GCC to avoid a relocation error with
# old binutils from base image. And modern GCC is required for binutils >2.41.
FROM gnu-build AS binutils
ARG BINUTILS_VERSION=2.44
ARG BINUTILS_SHA256=67be9198476cc37436e2801de649f4ad80bf0d02430d86aff63c6b59b6e23987

ADD --link --checksum=sha256:${BINUTILS_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/binutils/binutils-with-gold-${BINUTILS_VERSION}.tar.xz binutils.tar.xz
COPY --from=binutils-initial --link /build/out/toolchain /toolchain/
COPY --from=gcc-10 --link /build/out-install/toolchain /toolchain/
COPY --link scripts/binutils-build.sh /build/binutils-build.sh
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    BUILD_CC=/toolchain/bin/gcc \
    BUILD_CXX=/toolchain/bin/g++ \
    EXTRA_CONFIGURE_ARGS=--enable-bootstrap \
    /build/binutils-build.sh

FROM gnu-build AS gcc-15
ARG PARALLEL=16
ARG GCC_VERSION=15.2.0
ARG GCC_SHA256=438fd996826b0c82485a29da03a72d71d6e3541a83ec702df4271f6fe025d24e
ARG GMP_VERSION=6.2.1
ARG GMP_SHA256=fd4829912cddd12f84181c3451cc752be224643e87fac497b69edddadc49b4f2
ARG ISL_VERSION=0.24
ARG ISL_SHA256=fcf78dd9656c10eb8cf9fbd5f59a0b6b01386205fe1934b3b287a0a1898145c0
ARG MPC_VERSION=1.2.1
ARG MPC_SHA256=17503d2c395dfcf106b622dc142683c1199431d095367c6aacba6eec30340459
ARG MPFR_VERSION=4.1.0
ARG MPFR_SHA256=feced2d430dd5a97805fa289fed3fc8ff2b094c02d05287fd6133e7f1f0ec926

ADD --link --checksum=sha256:${GCC_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.xz gcc-15.tar.xz
ADD --link --checksum=sha256:${GMP_SHA256} --chmod=644 https://ftpmirror.gnu.org/gnu/gmp/gmp-${GMP_VERSION}.tar.xz gmp-6.2.tar.xz
ADD --link --checksum=sha256:${ISL_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/isl-${ISL_VERSION}.tar.bz2 isl-0.24.tar.bz2
ADD --link --checksum=sha256:${MPC_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/mpc-${MPC_VERSION}.tar.gz mpc-1.2.tar.gz
ADD --link --checksum=sha256:${MPFR_SHA256} --chmod=644 https://gcc.gnu.org/pub/gcc/infrastructure/mpfr-${MPFR_VERSION}.tar.bz2 mpfr-4.1.tar.bz2
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=gcc-10 --link /build/out-install/toolchain /toolchain/
COPY --link scripts/gcc-build-linux.sh /build/gcc-build.sh
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    GCC_VERSION=15 GMP_VERSION=6.2 ISL_VERSION=0.24 MPC_VERSION=1.2 MPFR_VERSION=4.1 \
    BUILD_CC=/toolchain/bin/gcc \
    BUILD_CXX=/toolchain/bin/g++ \
    PARALLEL=${PARALLEL} \
    /build/gcc-build.sh

FROM ${BASE_IMAGE} AS openssl
ARG PARALLEL=16
ARG OPENSSL_VERSION=3.0.16
ARG OPENSSL_SHA256=57e03c50feab5d31b152af2b764f10379aecd8ee92f16c985983ce4a99f7ef86
RUN apt-get install --force-yes build-essential
ADD --link --checksum=sha256:${OPENSSL_SHA256} --chmod=644 https://github.com/openssl/openssl/releases/download/openssl-${OPENSSL_VERSION}/openssl-${OPENSSL_VERSION}.tar.gz openssl.tar.gz
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=gcc-10 --link /build/out-install/toolchain /toolchain/
COPY --link scripts/openssl-build.sh /build/
RUN PARALLEL=${PARALLEL} /build/openssl-build.sh

FROM ${BASE_IMAGE} AS zstd
ARG PARALLEL=16
ARG ZSTD_VERSION=1.5.7
ARG ZSTD_SHA256=eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3
RUN apt-get install --force-yes build-essential
ADD --link --checksum=sha256:${ZSTD_SHA256} https://github.com/facebook/zstd/releases/download/v${ZSTD_VERSION}/zstd-${ZSTD_VERSION}.tar.gz zstd.tar.gz
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=gcc-10 --link /build/out-install/toolchain /toolchain/
COPY --link scripts/zstd-build.sh /build/zstd-build.sh
RUN PARALLEL=${PARALLEL} /build/zstd-build.sh

FROM ${BASE_IMAGE} AS cpython
ARG PARALLEL=16
ARG PYTHON_VERSION=3.13.3
ARG PYTHON_SHA256=40f868bcbdeb8149a3149580bb9bfd407b3321cd48f0be631af955ac92c0e041
ADD --link --checksum=sha256:${PYTHON_SHA256} https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tar.xz cpython.tar.xz
RUN apt-get install --force-yes autoconf automake build-essential libffi-dev make tar xz-utils zlib1g-dev
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=openssl --link /build/out/toolchain /toolchain/
COPY --from=gcc-10 --link /build/out-install/toolchain /toolchain/
COPY --link scripts/cpython-build.sh /build/
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    PARALLEL=${PARALLEL} /build/cpython-build.sh

FROM scratch AS download-llvm
ARG LLVM_VERSION=20.1.4
ARG LLVM_SHA256=a95365b02536ed4aef29b325c205dd89c268cba41503ab2fc05f81418613ab63
ADD --link --checksum=sha256:${LLVM_SHA256} https://github.com/llvm/llvm-project/releases/download/llvmorg-${LLVM_VERSION}/llvm-project-${LLVM_VERSION}.src.tar.xz /llvm.tar.xz

FROM scratch AS download-cmake-aarch64
ARG CMAKE_VERSION=3.31.7
ARG CMAKE_SHA256=e5b2dc2aefdca10afe09c8fa4ee2bbb4e732665943a94322f99c118781910c3c
ADD --link --checksum=sha256:${CMAKE_SHA256} https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-aarch64.tar.gz /cmake.tar.gz

FROM scratch AS download-cmake-x86_64
ARG CMAKE_VERSION=3.31.7
ARG CMAKE_SHA256=14e15d0b445dbeac686acc13fe13b3135e8307f69ccf4c5c91403996ce5aa2d4
ADD --link --checksum=sha256:${CMAKE_SHA256} https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz /cmake.tar.gz

FROM ${BASE_IMAGE} AS download-cmake
COPY --from=download-cmake-aarch64 --link /cmake.tar.gz /cmake.aarch64.tar.gz
COPY --from=download-cmake-x86_64 --link /cmake.tar.gz /cmake.x86_64.tar.gz
RUN <<EOF
if [ $(uname -m) = "x86_64" ]; then
  cp /cmake.x86_64.tar.gz /cmake.tar.gz
else
  cp /cmake.aarch64.tar.gz /cmake.tar.gz
fi
EOF

FROM scratch AS download-ninja-aarch64
ARG NINJA_VERSION=1.12.1
ARG NINJA_SHA256=5c25c6570b0155e95fce5918cb95f1ad9870df5768653afe128db822301a05a1
ADD --link --checksum=sha256:${NINJA_SHA256} https://github.com/ninja-build/ninja/releases/download/v${NINJA_VERSION}/ninja-linux-aarch64.zip /ninja.zip

FROM scratch AS download-ninja-x86_64
ARG NINJA_VERSION=1.12.1
ARG NINJA_SHA256=6f98805688d19672bd699fbbfa2c2cf0fc054ac3df1f0e6a47664d963d530255
ADD --link --checksum=sha256:${NINJA_SHA256} https://github.com/ninja-build/ninja/releases/download/v${NINJA_VERSION}/ninja-linux.zip /ninja.zip

FROM ${BASE_IMAGE} AS download-ninja
COPY --from=download-ninja-aarch64 /ninja.zip /ninja.aarch64.zip
COPY --from=download-ninja-x86_64 /ninja.zip /ninja.x86_64.zip
RUN <<EOF
if [ $(uname -m) = "x86_64" ]; then
  cp /ninja.x86_64.zip /ninja.zip
else
  cp /ninja.aarch64.zip /ninja.zip
fi
EOF

FROM ${BASE_IMAGE} AS clang-bootstrap
ARG PARALLEL_NINJA=64
ARG NINJA_MAX_LOAD=32

RUN <<EOF
apt-get install --force-yes libc6-dev patch tar xz-utils unzip zlib1g-dev
if [ $(uname -m) = "x86_64" ]; then
  apt-get install --force-yes libc6-dev:i386
fi
EOF
COPY --from=download-cmake --link /cmake.tar.gz cmake.tar.gz
COPY --from=download-ninja --link /ninja.zip ninja.zip
COPY --from=download-llvm --link /llvm.tar.xz llvm.tar.xz
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=gcc-15 --link /build/out-install/toolchain /toolchain/
COPY --from=cpython --link /build/out/toolchain /toolchain/
COPY --link scripts/clang-bootstrap.sh /build/
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
    PARALLEL_NINJA=${PARALLEL_NINJA} NINJA_MAX_LOAD=${NINJA_MAX_LOAD} \
    /build/clang-bootstrap.sh

FROM ${BASE_IMAGE} AS llvm-gnu-linux
ARG PARALLEL_NINJA=64
ARG NINJA_MAX_LOAD=32

# We need compiler-rt to find the libc statically linked object files (like crt1.o)
# for i386. Getting this to work reliably is finicky. Using a multiarch system
# with the multiarch install of i386 makes this "just work."
RUN <<EOF
apt-get install --force-yes libc6-dev patch tar xz-utils unzip zlib1g-dev
if [ $(uname -m) = "x86_64" ]; then
  apt-get install --force-yes libc6-dev:i386
fi
EOF
COPY --from=download-cmake --link /cmake.tar.gz cmake.tar.gz
COPY --from=download-ninja --link /ninja.zip ninja.zip
COPY --from=download-llvm --link /llvm.tar.xz llvm.tar.xz
COPY --from=binutils --link /build/out/toolchain /toolchain/
COPY --from=gcc-15 --link /build/out-support/ /toolchain/
COPY --from=cpython --link /build/out/toolchain /toolchain/
COPY --from=clang-bootstrap --link /build/out/toolchain /toolchain/
COPY --from=zstd --link /build/out/toolchain /toolchain/
COPY --link scripts/clang-gnu-stage2.sh /build/
RUN --mount=type=secret,id=secrets,target=/build/secrets,uid=1000 \
     PARALLEL_NINJA=${PARALLEL_NINJA} NINJA_MAX_LOAD=${NINJA_MAX_LOAD} \
    /build/clang-gnu-stage2.sh

FROM bullseye AS llvm-gnu-linux-x86-64
RUN apt-get install tar zstd
COPY --from=gcc-15 --link /build/out-support /build/gcc-support/
COPY --from=llvm-gnu-linux --link /build/out/toolchain/ /build/llvm/
COPY --link scripts/llvm-assemble-toolchain.sh /build/
ARG SOURCE_DATE_EPOCH
RUN <<EOF
if [ $(uname -m) = "x86_64" ]; then
  GNU_TARGET=x86_64-linux-gnu
else
  GNU_TARGET=aarch64-linux-gnu
fi

GNU_TARGET=${GNU_TARGET} GCC_VERSION=15 /build/llvm-assemble-toolchain.sh
EOF
