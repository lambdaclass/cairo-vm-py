FROM quay.io/pypa/manylinux2014_x86_64

# Install rust
RUN yum update && \
    yum install curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$HOME/.cargo/bin:$PATH"

# Install maturin with pip
RUN yum install -y python3-pip
RUN pip3 install tomli setuptools_rust
RUN pip3 install maturin

# Clone a repository
RUN yum install git
RUN git clone https://github.com/lambdaclass/cairo-rs-py.git

# Build your project
RUN cd cairo-rs-py && maturin build --release
