FROM node:12-debian

WORKDIR /screeps

COPY docker-entrypoint.sh /screeps/

RUN apt-get update && apt-get install -y --no-install-recommends \
    python2 build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN npm install screeps

ENTRYPOINT ["/screeps/docker-entrypoint.sh"]

CMD ["bash"]