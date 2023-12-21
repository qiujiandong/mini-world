FROM debian:bullseye-slim

COPY tsinghua_sources.list /etc/apt/sources.list.d/

RUN apt-get update && apt-get install -y --no-install-recommends \
    apt-transport-https ca-certificates
RUN apt-get update && apt-get install -y --no-install-recommends nodejs npm \
    && rm -rf /var/lib/apt/lists/*

CMD ["node"]