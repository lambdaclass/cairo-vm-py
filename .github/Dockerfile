FROM quay.io/pypa/manylinux2014_x86_64

# Install rust
RUN apt update && \
    apt install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    export PATH="$HOME/.cargo/bin:$PATH"

# Install maturin with pip
RUN apt install -y python3.9-pip
RUN pip3 install maturin

# Clone a repository
RUN apt install -y git
RUN git clone https://github.com/lambdaclass/cairo-rs-py.git

# Build your project
RUN cd cairo-rs-py && maturin build --release
