FROM ubuntu:latest

WORKDIR /app
ENV PATH=$PATH:/app/bin

RUN apt-get update && \
    apt-get -y --no-install-recommends install libpq5 libssl1.1 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY ./bins/the-orbacle /app/bin/the-orbacle

CMD ["the-orbacle"]
