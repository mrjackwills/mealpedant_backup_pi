FROM alpine:3.17

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add tzdata \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone 

WORKDIR /app

RUN mkdir /backups /logs \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app /backups /logs

USER ${DOCKER_APP_USER}

# This gets automatically updated via create_release.sh
#RUN wget https://github.com/mrjackwills/mealpedant_backup_pi/releases/download/v0.1.9/mealpedant_backup_pi_linux_armv6.tar.gz \
#	&& tar xzvf mealpedant_backup_pi_linux_armv6.tar.gz mealpedant_backup_pi && rm mealpedant_backup_pi_linux_armv6.tar.gz \
#	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app/


COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} mealpedant_backup_pi /app

CMD [ "/app/mealpedant_backup_pi"]