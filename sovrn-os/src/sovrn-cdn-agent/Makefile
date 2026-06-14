.PHONY: build test clean

build:
	CGO_ENABLED=0 go build -o ../../build/bin/sovrn-cdn-agent ./cmd/sovrn-cdn-agent/

test:
	go test ./... -v

clean:
	rm -f ../../build/bin/sovrn-cdn-agent

lint:
	golangci-lint run ./...

run: build
	./../../build/bin/sovrn-cdn-agent --config /etc/sovrn/cdn-agent.toml
