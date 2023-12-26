FROM miniball/node:12-debian

WORKDIR /screeps

COPY docker-entrypoint.sh /screeps/

RUN apt-get update && apt-get install -y --no-install-recommends \
    python2 build-essential vim\
    && rm -rf /var/lib/apt/lists/*

RUN npm install screeps
RUN npm install screepsmod-auth screepsmod-history screepsmod-mongo screepsmod-admin-utils screepsmod-features

ENTRYPOINT ["/screeps/docker-entrypoint.sh"]

CMD ["bash"]