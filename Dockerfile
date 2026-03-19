#########
# SETUP #
#########

FROM alpine:3 AS setup

ARG DOCKER_GUID=1000 \
    DOCKER_UID=1000 \
    DOCKER_APP_USER=app_user \
    DOCKER_APP_GROUP=app_group

ENV VIRT=".build_packages"

WORKDIR /app

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
    && adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
    && apk --no-cache add --virtual ${VIRT} ca-certificates \
    && apk del ${VIRT} \
    && mkdir /backups \
    && chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /backups

# This gets automatically updated via create_release.sh
ARG CURRENT_VERSION=v0.3.0

# Somewhat convoluted way to automatically select & download the correct package
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        x86_64) SUFFIX=x86_musl ;; \
        aarch64) SUFFIX=aarch64_musl ;; \
        *) exit 1 ;; \
    esac \ 
    && wget "https://github.com/mrjackwills/mealpedant_backup_pi/releases/download/${CURRENT_VERSION}/mealpedant_backup_pi_${SUFFIX}.tar.gz" \
    && tar xzvf "mealpedant_backup_pi_${SUFFIX}.tar.gz" mealpedant_backup_server \
    && rm "mealpedant_backup_pi_${SUFFIX}.tar.gz"

##########
# RUNNER #
##########

FROM scratch

ARG DOCKER_APP_USER=app_user \
    DOCKER_APP_GROUP=app_group

COPY --from=setup /app/ /app
COPY --from=setup /etc/group /etc/passwd /etc/
COPY --from=setup /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=setup --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /backups /backups

USER ${DOCKER_APP_USER}

ENTRYPOINT ["/app/mealpedant_backup_pi"]
