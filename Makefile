SRC = $(shell find src/ -type f -name '*.rs')

default:	target/functions/lambda

target/functions/lambda:	$(SRC) Dockerfile
	@docker build -t sailfishos-chum-web .
	@docker create --name sailfishos-chum-web sailfishos-chum-web /bin/sh
	@docker cp sailfishos-chum-web:/target .
	@docker rm sailfishos-chum-web
	@docker image rm sailfishos-chum-web

.PHONY:	deploy
deploy:	target/functions/lambda
	netlify deploy

.PHONY:	deploy/prod
deploy/prod:	target/functions/lambda
	netlify deploy --prod
