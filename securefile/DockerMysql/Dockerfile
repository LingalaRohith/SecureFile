FROM mysql:latest

ENV MYSQL_ROOT_PASSWORD=root
# ENV MYSQL_USER=root

COPY Data.sql /docker-entrypoint-initdb.d/

EXPOSE 3306
