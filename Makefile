.PHONY: all frontend backend dev deploy

all: frontend backend

frontend:
	cd frontend && npm install && npm run build

backend:
	cargo build

dev:
	tmux \
		new-session 'cd frontend && npm install && npm run dev' \; \
		split-window -h 'cargo watch -w src -x "run -- --reverse-proxy-url http://localhost:3000"' \;

dev-debug:
	tmux \
		new-session 'cd frontend && npm install && npm run dev' \; \
		split-window -h 'cargo watch -w src -x "run -- -d --reverse-proxy-url http://localhost:3000"' \;

deploy:
	./scripts/deploy.sh