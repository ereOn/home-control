.PHONY: all frontend backend dev deploy

all: frontend backend

frontend:
	cd frontend && npm install && npm run build

backend:
	cargo build

dev:
	tmux \
		new-session 'cd frontend && npm install && npm run dev' \; \
		split-window -h 'cargo run' \;

deploy:
	./scripts/deploy.sh