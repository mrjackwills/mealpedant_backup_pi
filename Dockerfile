#########
# SETUP #
#########

FROM alpine:3.20 AS setup

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
ARG MEALPEDANT_BACKUP_PI_VERSION=v0.1.14

RUN wget "https://github.com/mrjackwills/mealpedant_backup_pi/releases/download/${MEALPEDANT_BACKUP_PI_VERSION}/mealpedant_backup_pi_linux_armv6.tar.gz" \
	&& tar xzvf mealpedant_backup_pi_linux_armv6.tar.gz mealpedant_backup_pi \
	&& rm mealpedant_backup_pi_linux_armv6.tar.gz \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app/

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
