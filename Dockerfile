FROM rust:alpine3.21

WORKDIR /General-Api

# idunno how tf they expected for me to find this line
RUN apk add --no-cache musl-dev

#TODO: in production, delete all the residual files except for the executable and it's dependencies
COPY ./target/release/general-api .


EXPOSE 5050/tcp


CMD ./target/release/general-api



