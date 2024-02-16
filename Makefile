TAG?=latest
NAME:=healthcat-rs
DOCKER_REPOSITORY:=registry.gitlab.casa-systems.com/dimitar.georgievski
DOCKER_IMAGE_NAME:=$(DOCKER_REPOSITORY)/$(NAME)
GIT_COMMIT:=$(shell git describe --dirty --always)
VERSION:=$(shell /usr/local/bin/toml get Cargo.toml package.version | tr -d '"')
EXTRA_RUN_ARGS?=

build:
	cargo build 

run:
	cargo run 

build-docker:
	cargo check
	docker build -t $(DOCKER_IMAGE_NAME):$(VERSION) .

push-docker:
	docker push $(DOCKER_IMAGE_NAME):$(VERSION)

version-get:
	@echo "Current version $(VERSION)"

version-set:
	@next="$(TAG)" && \
	current="$(VERSION)" && \
	/usr/bin/sed -i "s/^version = \"$$current\"/version = \"$$next\"/g" Cargo.toml && \
	/usr/bin/sed -i "s/$$current/$$next/g" deploy/kustomize/overlays/cicd/kustomization.yaml && \
	echo "Version $$next set in code and kustomize manifests"

deploy:
	@olddir=`pwd` 
	cd deploy/kustomize 
	kubectl apply -k overlays/cicd/
	cd $$olddir