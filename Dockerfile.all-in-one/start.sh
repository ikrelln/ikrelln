docker-entrypoint.sh postgres &
sleep 5
envsubst '\$ZIPKIN_BASE_URL' < /etc/nginx/conf.d/zipkin.conf.template > /etc/nginx/nginx.conf
nginx &
/usr/local/bin/diesel setup
/usr/local/bin/ikrelln
