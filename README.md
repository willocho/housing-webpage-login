# housing-webpage-login

This application consists of a simple web app and server. The server is written in Rust, using the rocket and sqlx packages. The client is a react application written in typescript, built using vite.

## Client Commands

### Building the Client
Build the client using the `npm run build` command.

### Running a Local Dev Server
You can run a local dev server using the `npm run dev` command. The dev command is set up to proxy api requests to `http://localhost:8000` (see `webpage/vite.config.ts`, so you want to make sure that the application server is also running.

## Server Commands

### Running the server
You can run the application server using the `cargo run` command.
