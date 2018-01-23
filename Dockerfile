FROM rust:1.23.0
WORKDIR ./
COPY . .
ENV NB_CONNECTION 1
ENV DATABASE_URL test.sqlite
EXPOSE 8080
#RUN cargo install diesel_cli
#RUN diesel migration run
RUN cargo install --features sqlite
CMD [ "ikrelln" ]
