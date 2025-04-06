FROM rust:alpine3.21

WORKDIR /General-Api

# idunno how tf they expected for me to find this line
RUN apk add --no-cache musl-dev

COPY . .


RUN cargo build --release


EXPOSE 5000/tcp


CMD ./target/release/general-api



